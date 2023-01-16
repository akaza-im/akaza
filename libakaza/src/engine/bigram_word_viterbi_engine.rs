use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::env;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::{bail, Result};
use encoding_rs::UTF_8;
use log::{info, warn};

use crate::config::Config;
use crate::dict::loader::load_dicts;
use crate::dict::merge_dict::merge_dict;
use crate::dict::skk::read::read_skkdict;
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
    user_data: Option<Arc<Mutex<UserData>>>,
    load_user_config: bool,
    dicts: Option<HashMap<String, Vec<String>>>,
    single_term: Option<HashMap<String, Vec<String>>>,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(
        dicts: Option<HashMap<String, Vec<String>>>,
        single_term: Option<HashMap<String, Vec<String>>>,
    ) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
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
        let system_unigram_lm = MarisaSystemUnigramLM::load(
            Self::try_load("stats-vibrato-unigram.trie")?
                .to_string_lossy()
                .to_string()
                .as_str(),
        )?;
        let system_bigram_lm = MarisaSystemBigramLM::load(
            Self::try_load("stats-vibrato-bigram.trie")?
                .to_string_lossy()
                .to_string()
                .as_str(),
        )?;
        let system_dict = read_skkdict(Self::try_load("SKK-JISYO.akaza")?.as_path(), UTF_8)?;

        let user_data = if let Some(d) = &self.user_data {
            d.clone()
        } else {
            Arc::new(Mutex::new(UserData::default()))
        };

        let config = if self.load_user_config {
            self.load_config()?
        } else {
            Config::default()
        };

        let dict = load_dicts(&config.dicts)?;
        let dict = merge_dict(vec![system_dict, dict]);
        let dict = if let Some(dd) = &self.dicts {
            merge_dict(vec![dict, dd.clone()])
        } else {
            dict
        };

        let single_term = if let Some(st) = &config.single_term {
            load_dicts(st)?
        } else {
            HashMap::new()
        };
        let single_term = if let Some(dd) = &self.single_term {
            merge_dict(vec![single_term, dd.clone()])
        } else {
            single_term
        };

        // 辞書を元に、トライを作成していく。
        let mut kana_trie = CedarwoodKanaTrie::default();
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
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
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

    pub fn try_load(file_name: &str) -> Result<PathBuf> {
        if cfg!(test) {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"));
            let path = path.join("../akaza-data/data/").join(file_name);
            if path.exists() {
                Ok(path)
            } else {
                bail!("There's no {} for testing.", path.to_string_lossy(),)
            }
        } else if let Ok(dir) = env::var("AKAZA_DATA_DIR") {
            let dir = Path::new(dir.as_str());
            let file = dir.join(file_name);
            if file.exists() {
                Ok(file)
            } else {
                bail!(
                    "There's no {} in AKAZA_DATA_DIR({:?})",
                    file.to_string_lossy(),
                    dir,
                )
            }
        } else {
            let path = xdg::BaseDirectories::with_prefix("akaza")?.find_data_file(file_name);
            if let Some(path) = path {
                Ok(path)
            } else {
                bail!("There's no {} in XDG_DATA_DIRS", file_name)
            }
        }
    }
}
