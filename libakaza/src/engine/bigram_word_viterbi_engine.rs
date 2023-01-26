use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::config::{DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};
use crate::dict::loader::{load_dicts, load_dicts_with_cache};
use crate::engine::base::HenkanEngine;
use crate::graph::candidate::Candidate;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::GraphResolver;
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji::base::KanaKanjiDict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::user_side_data::user_data::UserData;

/// バイグラムのビタビベースかな漢字変換エンジンです。
/// 単語バイグラムを採用しています。
pub struct BigramWordViterbiEngine<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> {
    graph_builder: GraphBuilder<U, B, KD>,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    pub user_data: Arc<Mutex<UserData>>,
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> HenkanEngine
    for BigramWordViterbiEngine<U, B, KD>
{
    fn learn(&mut self, candidates: &[Candidate]) {
        self.user_data.lock().unwrap().record_entries(candidates);
    }

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> Result<Vec<Vec<Candidate>>> {
        let lattice = self.to_lattice(yomi, force_ranges)?;
        self.resolve(&lattice)
    }
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> BigramWordViterbiEngine<U, B, KD> {
    pub fn resolve(&self, lattice: &LatticeGraph<U, B>) -> Result<Vec<Vec<Candidate>>> {
        self.graph_resolver.resolve(lattice)
    }

    pub fn to_lattice(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> Result<LatticeGraph<U, B>> {
        let segmentation_result = &self.segmenter.build(yomi, force_ranges);
        let lattice = self.graph_builder.construct(yomi, segmentation_result);
        Ok(lattice)
    }
}

pub struct BigramWordViterbiEngineBuilder {
    user_data: Option<Arc<Mutex<UserData>>>,
    load_user_config: bool,
    config: EngineConfig,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(config: EngineConfig) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
            user_data: None,
            load_user_config: false,
            config,
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
    ) -> Result<
        BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM, MarisaKanaKanjiDict>,
    > {
        let model_name = self.config.model.clone();

        let system_unigram_lm =
            MarisaSystemUnigramLM::load(Self::try_load(&model_name, "unigram.model")?.as_str())?;
        let system_bigram_lm =
            MarisaSystemBigramLM::load(Self::try_load(&model_name, "bigram.model")?.as_str())?;
        let system_dict = Self::try_load(&model_name, "SKK-JISYO.akaza")?;

        let user_data = if let Some(d) = &self.user_data {
            d.clone()
        } else {
            Arc::new(Mutex::new(UserData::default()))
        };

        let dict = {
            let mut dicts = self
                .config
                .dicts
                .iter()
                .filter(|it| it.usage == DictUsage::Normal)
                .cloned()
                .collect::<Vec<_>>();
            dicts.push(DictConfig {
                path: system_dict,
                dict_type: DictType::SKK,
                encoding: DictEncoding::Utf8,
                usage: DictUsage::Normal,
            });

            if self.config.dict_cache {
                load_dicts_with_cache(&dicts, "kana_kanji_cache.marisa")?
            } else {
                let dict = load_dicts(&dicts)?;
                MarisaKanaKanjiDict::build(dict)?
            }
        };

        let single_term = {
            let dicts = self
                .config
                .dicts
                .iter()
                .filter(|it| it.usage == DictUsage::SingleTerm)
                .cloned()
                .collect::<Vec<_>>();
            if self.config.dict_cache {
                load_dicts_with_cache(&dicts, "single_term_cache.marisa")?
            } else {
                let dict = load_dicts(&dicts)?;
                MarisaKanaKanjiDict::build(dict)?
            }
        };

        // 辞書を元に、トライを作成していく。
        let mut kana_trie = CedarwoodKanaTrie::default();
        for yomi in dict.yomis() {
            assert!(!yomi.is_empty());
            kana_trie.update(yomi.as_str());
        }
        for yomi in single_term.yomis() {
            assert!(!yomi.is_empty());
            kana_trie.update(yomi.as_str());
        }

        let segmenter = Segmenter::new(vec![
            Arc::new(Mutex::new(kana_trie)),
            user_data.lock().unwrap().kana_trie.clone(),
        ]);

        let graph_builder: GraphBuilder<
            MarisaSystemUnigramLM,
            MarisaSystemBigramLM,
            MarisaKanaKanjiDict,
        > = GraphBuilder::new(
            dict,
            single_term,
            user_data.clone(),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let graph_resolver = GraphResolver::default();

        Ok(BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            user_data,
        })
    }

    fn try_load(model_dir: &str, name: &str) -> Result<String> {
        Ok(model_dir.to_string() + "/" + name)
    }
}
