use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::{bail, Result};
use encoding_rs::{EUC_JP, UTF_8};
use log::{error, info, warn};

use crate::config::{Config, DictConfig};
use crate::engine::base::HenkanEngine;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::{Candidate, GraphResolver};
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji_dict::KanaKanjiDict;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::romkan::RomKanConverter;
use crate::skk::ari2nasi::Ari2Nasi;
use crate::skk::merge_skkdict::merge_skkdict;
use crate::skk::skkdict::parse_skkdict;
use crate::user_side_data::user_data::UserData;

pub struct SystemDataLoader {
    pub system_unigram_lm: MarisaSystemUnigramLM,
    pub system_bigram_lm: MarisaSystemBigramLM,
    pub system_kana_kanji_dict: KanaKanjiDict,
    pub system_single_term_dict: KanaKanjiDict,
    pub system_kana_trie: MarisaKanaTrie,
}

impl SystemDataLoader {
    pub fn load(system_data_dir: &str) -> Result<SystemDataLoader> {
        let system_unigram_lm = MarisaSystemUnigramLM::load(
            (system_data_dir.to_string() + "/stats-vibrato-unigram.trie").as_str(),
        )?;
        let system_bigram_lm = MarisaSystemBigramLM::load(
            (system_data_dir.to_string() + "/stats-vibrato-bigram.trie").as_str(),
        )?;

        let system_kana_kanji_dict =
            KanaKanjiDict::load((system_data_dir.to_string() + "/system_dict.trie").as_str())?;
        let system_single_term_dict =
            KanaKanjiDict::load((system_data_dir.to_string() + "/single_term.trie").as_str())?;
        let system_kana_trie =
            MarisaKanaTrie::load((system_data_dir.to_string() + "/kana.trie").as_str())?;

        Ok(SystemDataLoader {
            system_unigram_lm,
            system_bigram_lm,
            system_kana_kanji_dict,
            system_single_term_dict,
            system_kana_trie,
        })
    }
}

/// バイグラムのビタビベースかな漢字変換エンジンです。
/// 単語バイグラムを採用しています。
pub struct BigramWordViterbiEngine<U: SystemUnigramLM, B: SystemBigramLM> {
    graph_builder: GraphBuilder<U, B>,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    romkan_converter: RomKanConverter,
    pub user_data: Arc<Mutex<UserData>>,
}

impl<U: SystemUnigramLM, B: SystemBigramLM> HenkanEngine for BigramWordViterbiEngine<U, B> {
    fn learn(&mut self, candidates: &[Candidate]) {
        self.user_data.lock().unwrap().record_entries(candidates);
    }

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<VecDeque<Candidate>>> {
        // 先頭が大文字なケースと、URL っぽい文字列のときは変換処理を実施しない。
        if (!yomi.is_empty()
            && yomi.chars().next().unwrap().is_ascii_uppercase()
            && (force_ranges.is_none()
                || (force_ranges.is_none() && force_ranges.unwrap().is_empty())))
            || yomi.starts_with("https://")
            || yomi.starts_with("http://")
        {
            return Ok(vec![VecDeque::from([Candidate::new(yomi, yomi, 0_f32)])]);
        }

        let lattice = self.to_lattice(yomi, force_ranges)?;
        self.resolve(&lattice)
    }
}

impl<U: SystemUnigramLM, B: SystemBigramLM> BigramWordViterbiEngine<U, B> {
    pub fn resolve(&self, lattice: &LatticeGraph<U, B>) -> Result<Vec<VecDeque<Candidate>>> {
        self.graph_resolver.resolve(lattice)
    }

    pub fn to_lattice(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> Result<LatticeGraph<U, B>> {
        // ローマ字からひらがなへの変換をする。
        let yomi = self.romkan_converter.to_hiragana(yomi);

        /*
            TODO: C++ 版 akaza では子音を先に取り除いておいて、あとからまたくっつけるという処理をしていたようだが、
            これをやる意味が今はわからないので一旦あとまわし。

                // 子音だが、N は NN だと「ん」になるので処理しない。
        std::string consonant;
        {
            std::wregex trailing_consonant(cnv.from_bytes(R"(^(.*?)([qwrtypsdfghjklzxcvbm]+)$)"));
            std::wsmatch sm;
            if (std::regex_match(whiragana, sm, trailing_consonant)) {
                hiragana = cnv.to_bytes(sm.str(1));
                consonant = cnv.to_bytes(sm.str(2));
                D(std::cout << "CONSONANT=" << consonant << std::endl);
            }
        }

        Graph graph = graphResolver_->graph_construct(cnv.from_bytes(hiragana), forceSelectedClauses);
        graphResolver_->fill_cost(graph);
        D(graph.dump());
        std::vector<std::vector<std::shared_ptr<akaza::Node>>> nodes = graphResolver_->find_nbest(graph);
        if (consonant.empty()) {
            return nodes;
        } else {
            D(std::cout << " Adding Consonant=" << consonant << std::endl);
            nodes.push_back({{
                                     akaza::create_node(
                                             graphResolver_->system_unigram_lm_,
                                             src.size(),
                                             cnv.from_bytes(consonant),
                                             cnv.from_bytes(consonant)
                                     )
                             }});
            return nodes;
        }
             */

        let self1 = &self.segmenter;
        let segmentation_result = self1.build(yomi.as_str(), force_ranges);
        let lattice = self
            .graph_builder
            .construct(yomi.as_str(), segmentation_result);
        Ok(lattice)
    }
}

pub struct BigramWordViterbiEngineBuilder {
    system_data_dir: String,
    load_user_config: bool,
    user_data: Option<Arc<Mutex<UserData>>>,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(system_data_dir: &str) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
            system_data_dir: system_data_dir.to_string(),
            load_user_config: false,
            user_data: None,
        }
    }

    pub fn load_user_config(&mut self, load_user_config: bool) -> &mut Self {
        self.load_user_config = load_user_config;
        self
    }

    pub fn user_data(&mut self, user_data: Arc<Mutex<UserData>>) -> &mut Self {
        self.user_data = Some(user_data);
        self
    }

    pub fn build(
        &self,
    ) -> Result<BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM>> {
        let system_data_loader = SystemDataLoader::load(self.system_data_dir.as_str())?;

        let user_data = if let Some(d) = &self.user_data {
            d.clone()
        } else {
            Arc::new(Mutex::new(UserData::default()))
        };

        {
            let t1 = SystemTime::now();
            let config = if self.load_user_config {
                self.load_config()?
            } else {
                Config::default()
            };
            let dicts = self.load_dicts(config)?;
            // 次に、辞書を元に、トライを作成していく。
            let yomis = dicts.keys();
            let mut kana_trie = CedarwoodKanaTrie::default();
            for yomi in yomis {
                kana_trie.update(yomi.as_str());
            }
            let t2 = SystemTime::now();
            info!(
                "Loaded configuration in {}msec.",
                t2.duration_since(t1).unwrap().as_millis()
            );
        }

        let segmenter = Segmenter::new(vec![
            Arc::new(Mutex::new(system_data_loader.system_kana_trie)),
            user_data.lock().unwrap().kana_trie.clone(),
        ]);

        let graph_builder = GraphBuilder::new_with_default_score(
            system_data_loader.system_kana_kanji_dict,
            system_data_loader.system_single_term_dict,
            user_data.clone(),
            Rc::new(system_data_loader.system_unigram_lm),
            Rc::new(system_data_loader.system_bigram_lm),
        );

        let graph_resolver = GraphResolver::default();

        let romkan_converter = RomKanConverter::new();

        Ok(BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            romkan_converter,
            user_data,
        })
    }

    fn load_config(&self) -> anyhow::Result<Config> {
        let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
        let configfile = basedir.get_config_file("config.yml");
        let config = match Config::load_from_file(configfile.to_str().unwrap()) {
            Ok(config) => config,
            Err(err) => {
                warn!(
                    "Cannot load configuration file: {} {}",
                    configfile.to_string_lossy(),
                    err
                );
                return Ok(Config::default());
            }
        };
        info!(
            "Loaded config file: {}, {:?}",
            configfile.to_string_lossy(),
            config
        );
        Ok(config)
    }

    pub fn load_dicts(&self, config: Config) -> Result<HashMap<String, Vec<String>>> {
        let mut dicts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        for dict in config.dicts {
            match self.load_dict(&dict) {
                Ok(dict) => {
                    // TODO 辞書をうまく使う
                    dicts.push(dict);
                }
                Err(err) => {
                    error!("Cannot load {:?}. {}", dict, err);
                    // 一顧の辞書の読み込みに失敗しても、他の辞書は読み込むべきなので
                    // 処理は続行する
                }
            }
        }
        Ok(merge_skkdict(dicts))
    }

    pub fn load_dict(&self, dict: &DictConfig) -> Result<HashMap<String, Vec<String>>> {
        info!(
            "Loading dictionary: {} {:?} {}",
            dict.path, dict.encoding, dict.dict_type
        );
        let encoding = match &dict.encoding {
            Some(encoding) => match encoding.to_ascii_lowercase().as_str() {
                "euc-jp" | "euc_jp" => EUC_JP,
                "utf-8" => UTF_8,
                _ => {
                    bail!(
                        "Unknown enconding in configuration: {} for {}",
                        encoding,
                        dict.path
                    )
                }
            },
            None => UTF_8,
        };

        let file = File::open(dict.path.as_str())?;
        let mut buf: Vec<u8> = Vec::new();
        BufReader::new(file).read_to_end(&mut buf)?;
        let (src, _, _) = encoding.decode(buf.as_slice());
        match dict.dict_type.as_str() {
            "skk" => {
                let (ari, nasi) = parse_skkdict(src.to_string().as_str())?;
                let ari2nasi = Ari2Nasi::new();
                let ari = ari2nasi.ari2nasi(&ari)?;
                let merged = merge_skkdict(vec![ari, nasi]);
                info!("Loaded {}: {} entries.", dict.path, merged.len());
                Ok(merged)
            }
            _ => {
                bail!(
                    "Unknown dictionary type: {} for {}",
                    dict.dict_type,
                    dict.path
                );
            }
        }
    }
}
