use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::Result;
use log::{info, warn};

use crate::config::Config;
use crate::dict::loader::load_dicts;
use crate::dict::merge_dict::merge_dict;
use crate::engine::base::HenkanEngine;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::{Candidate, GraphResolver};
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::romkan::RomKanConverter;
use crate::user_side_data::user_data::UserData;

pub struct SystemDataLoader {
    pub system_unigram_lm: MarisaSystemUnigramLM,
    pub system_bigram_lm: MarisaSystemBigramLM,
}

impl SystemDataLoader {
    pub fn load(system_data_dir: &str) -> Result<SystemDataLoader> {
        let system_unigram_lm = MarisaSystemUnigramLM::load(
            (system_data_dir.to_string() + "/stats-vibrato-unigram.trie").as_str(),
        )?;
        let system_bigram_lm = MarisaSystemBigramLM::load(
            (system_data_dir.to_string() + "/stats-vibrato-bigram.trie").as_str(),
        )?;

        Ok(SystemDataLoader {
            system_unigram_lm,
            system_bigram_lm,
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
    ) -> Result<Vec<VecDeque<Candidate>>> {
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
    user_data: Option<Arc<Mutex<UserData>>>,
    load_user_config: bool,
    dicts: Option<HashMap<String, Vec<String>>>,
    single_term: Option<HashMap<String, Vec<String>>>,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(
        system_data_dir: &str,
        dicts: Option<HashMap<String, Vec<String>>>,
        single_term: Option<HashMap<String, Vec<String>>>,
    ) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
            system_data_dir: system_data_dir.to_string(),
            user_data: None,
            load_user_config: false,
            dicts,
            single_term,
        }
    }

    // TODO: ユーザー設定を読むかどうかの責任は、Engine ではなく EngineFactory 的なクラスを用意して
    // 責務を移管する。
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

        // TODO このへんごちゃごちゃしすぎ。
        let (dict, single_term, mut kana_trie) = {
            let t1 = SystemTime::now();
            let config = if self.load_user_config {
                self.load_config()?
            } else {
                Config::default()
            };
            let dicts = load_dicts(&config.dicts)?;
            let single_term = if let Some(st) = &config.single_term {
                load_dicts(st)?
            } else {
                HashMap::new()
            };
            // 次に、辞書を元に、トライを作成していく。
            let kana_trie = CedarwoodKanaTrie::default();
            let t2 = SystemTime::now();
            info!(
                "Loaded configuration in {}msec.",
                t2.duration_since(t1).unwrap().as_millis()
            );
            (dicts, single_term, kana_trie)
        };
        let dict = if let Some(dd) = &self.dicts {
            merge_dict(vec![dict, dd.clone()])
        } else {
            dict
        };
        let single_term = if let Some(dd) = &self.single_term {
            merge_dict(vec![single_term, dd.clone()])
        } else {
            single_term
        };
        for yomi in dict.keys() {
            assert!(!yomi.is_empty());
            kana_trie.update(yomi.as_str());
        }
        for yomi in single_term.keys() {
            assert!(!yomi.is_empty());
            kana_trie.update(yomi.as_str());
        }

        let segmenter = Segmenter::new(vec![
            Arc::new(Mutex::new(kana_trie)),
            user_data.lock().unwrap().kana_trie.clone(),
        ]);

        let graph_builder = GraphBuilder::new_with_default_score(
            dict,
            single_term,
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

    fn load_config(&self) -> Result<Config> {
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
}
