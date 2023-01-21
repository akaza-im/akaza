use std::collections::vec_deque::VecDeque;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::config::{Config, DictConfig};
use crate::dict::loader::load_dicts_ex;
use crate::engine::base::HenkanEngine;
use crate::graph::candidate::Candidate;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::GraphResolver;
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji::base::KanaKanjiDict;
use crate::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::resource::detect_resource_path;
use crate::romkan::RomKanConverter;
use crate::user_side_data::user_data::UserData;

/// バイグラムのビタビベースかな漢字変換エンジンです。
/// 単語バイグラムを採用しています。
pub struct BigramWordViterbiEngine<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> {
    graph_builder: GraphBuilder<U, B, KD>,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    romkan_converter: RomKanConverter,
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

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> BigramWordViterbiEngine<U, B, KD> {
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

        let segmentation_result = &self.segmenter.build(yomi.as_str(), force_ranges);
        let lattice = self
            .graph_builder
            .construct(yomi.as_str(), segmentation_result);
        Ok(lattice)
    }
}

pub struct BigramWordViterbiEngineBuilder {
    user_data: Option<Arc<Mutex<UserData>>>,
    load_user_config: bool,
    pub config: Config,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(config: Config) -> BigramWordViterbiEngineBuilder {
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
        BigramWordViterbiEngine<
            MarisaSystemUnigramLM,
            MarisaSystemBigramLM,
            HashmapVecKanaKanjiDict,
        >,
    > {
        let model_name = self
            .config
            .model
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let system_unigram_lm = MarisaSystemUnigramLM::load(
            Self::try_load(&format!("{}/unigram.model", model_name))?.as_str(),
        )?;
        let system_bigram_lm = MarisaSystemBigramLM::load(
            Self::try_load(&format!("{}/bigram.model", model_name))?.as_str(),
        )?;
        let system_dict = Self::try_load(&format!("{}/SKK-JISYO.akaza", model_name))?;

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
                .map(|it| *it.clone())
                .collect::<Vec<_>>();
            dicts.push(DictConfig {
                path: system_dict,
                dict_type: "skk".to_string(),
                encoding: None,
            });

            load_dicts_ex(&dicts, "kana_kanji_cache.marisa")?
        };

        let single_term = load_dicts_ex(&self.config.single_term, "single_term_cache.marisa")?;

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
        > = GraphBuilder::new_with_default_score(
            dict,
            single_term,
            user_data.clone(),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let graph_resolver = GraphResolver::default();

        let mapping_name = self
            .config
            .romkan
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let mapping_name = mapping_name.as_str();
        let romkan_converter = RomKanConverter::new(mapping_name)?;

        Ok(BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            romkan_converter,
            user_data,
        })
    }

    fn try_load(name: &str) -> Result<String> {
        detect_resource_path("model", "AKAZA_MODEL_DIR", name)
    }
}
