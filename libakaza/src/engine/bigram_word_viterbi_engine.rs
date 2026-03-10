use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{error, info};

use crate::config::{DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};
use crate::dict::loader::{load_dicts, load_dicts_with_cache};
use crate::engine::base::HenkanEngine;
use crate::graph::candidate::Candidate;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::{GraphResolver, KBestPath};
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::reranking::ReRankingWeights;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji::base::KanaKanjiDict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::lm::base::{SystemBigramLM, SystemSkipBigramLM, SystemUnigramLM};
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_skip_bigram::MarisaSystemSkipBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::user_side_data::user_data::UserData;

/// バイグラムのビタビベースかな漢字変換エンジンです。
/// 単語バイグラムを採用しています。
pub struct BigramWordViterbiEngine<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> {
    graph_builder: GraphBuilder<U, B, KD>,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    pub user_data: Arc<Mutex<UserData>>,
    reranking_weights: ReRankingWeights,
    skip_bigram_lm: Option<Rc<MarisaSystemSkipBigramLM>>,
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> Debug
    for BigramWordViterbiEngine<U, B, KD>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("BigramWordViterbiEngine"))
    }
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> HenkanEngine
    for BigramWordViterbiEngine<U, B, KD>
{
    fn learn(&mut self, candidates: &[Candidate]) {
        match self.user_data.lock() {
            Ok(mut user_data) => user_data.record_entries(candidates),
            Err(e) => error!("learn: failed to lock user_data: {}", e),
        }
    }

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> Result<Vec<Vec<Candidate>>> {
        // リランキングを適用するため、k-best で複数パスを取得してリランキング
        // k=10 で十分な候補パターンを取得し、リランキング後の最良パスを返す
        let paths = self.convert_k_best(yomi, force_ranges, 10)?;
        if let Some(best_path) = paths.first() {
            Ok(best_path.segments.clone())
        } else {
            Ok(vec![])
        }
    }

    fn convert_k_best(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
        k: usize,
    ) -> Result<Vec<KBestPath>> {
        let lattice = self.to_lattice(yomi, force_ranges)?;
        let mut paths = self.graph_resolver.resolve_k_best(&lattice, k)?;
        // skip-bigram コストは Viterbi DP 内で計算済み（GraphResolver 経由）
        self.reranking_weights.rerank(&mut paths);
        Ok(paths)
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
    config: EngineConfig,
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(config: EngineConfig) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
            user_data: None,
            config,
        }
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
        let skip_bigram_path = Self::try_load(&model_name, "skip_bigram.model")?;
        let skip_bigram_lm = match MarisaSystemSkipBigramLM::load(&skip_bigram_path) {
            Ok(lm) => {
                info!("Loaded skip-bigram model: {}", skip_bigram_path);
                Some(Rc::new(lm))
            }
            Err(_) => {
                info!(
                    "Skip-bigram model not found (optional): {}",
                    skip_bigram_path
                );
                None
            }
        };
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

        let reranking_weights = self.config.reranking_weights.clone();

        let graph_resolver = GraphResolver::new(
            skip_bigram_lm
                .clone()
                .map(|lm| lm as Rc<dyn SystemSkipBigramLM>),
            reranking_weights.skip_bigram_weight,
        );

        Ok(BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            user_data,
            reranking_weights,
            skip_bigram_lm,
        })
    }

    fn try_load(model_dir: &str, name: &str) -> Result<String> {
        let path = std::path::Path::new(model_dir).join(name);
        Ok(path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::graph_builder::GraphBuilder;
    use crate::graph::graph_resolver::GraphResolver;
    use crate::graph::segmenter::Segmenter;
    use crate::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
    use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
    use crate::lm::base::SystemUnigramLM;
    use crate::lm::system_bigram::MarisaSystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;
    use std::collections::HashMap;

    /// convert メソッドでリランキングが適用されることを確認
    #[test]
    fn test_convert_applies_reranking() -> anyhow::Result<()> {
        // リランキングによって順位が逆転するケースを設定
        // "このもでる" → Viterbi では "この/も/出る" が選ばれるが (3トークン)、
        // リランキングでは length_weight により "この/モデル" が選ばれる (2トークン)
        let dict = HashMap::from([
            ("この".to_string(), vec!["この".to_string()]),
            ("も".to_string(), vec!["も".to_string()]),
            ("でる".to_string(), vec!["出る".to_string()]),
            ("もでる".to_string(), vec!["モデル".to_string()]),
        ]);

        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.add("この/この", 1.0);
        unigram_builder.add("も/も", 0.5);
        unigram_builder.add("出る/でる", 1.0);
        unigram_builder.add("モデル/もでる", 1.5);
        unigram_builder.set_total_words(1000);
        unigram_builder.set_unique_words(100);
        let system_unigram_lm = unigram_builder.build()?;

        let unigram_map = system_unigram_lm.as_hash_map();
        let kono_id = unigram_map.get("この/この").unwrap().0;
        let mo_id = unigram_map.get("も/も").unwrap().0;
        let deru_id = unigram_map.get("出る/でる").unwrap().0;
        let model_id = unigram_map.get("モデル/もでる").unwrap().0;

        let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
        bigram_builder.set_default_edge_cost(5.0);
        // "この→も→出る" のバイグラムコスト (Viterbi で選ばれるように低く設定)
        bigram_builder.add(kono_id, mo_id, 0.5);
        bigram_builder.add(mo_id, deru_id, 0.5);
        // "この→モデル" のバイグラムコスト
        bigram_builder.add(kono_id, model_id, 2.0);
        let system_bigram_lm = bigram_builder.build()?;

        // エンジンを構築
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let kana_trie = CedarwoodKanaTrie::build(vec![
            "この".to_string(),
            "も".to_string(),
            "でる".to_string(),
            "もでる".to_string(),
        ]);

        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph_resolver = GraphResolver::default();

        let engine = BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            user_data: Arc::new(Mutex::new(UserData::default())),
            reranking_weights: ReRankingWeights::default(),
            skip_bigram_lm: None,
        };

        // convert を呼び出し（リランキング適用済み）
        let result = engine.convert("このもでる", None)?;

        // リランキングにより "この/モデル" が選ばれることを確認
        let text: Vec<String> = result
            .iter()
            .filter_map(|segment| segment.first().map(|c| c.surface.clone()))
            .collect();

        assert_eq!(text.join("/"), "この/モデル");

        Ok(())
    }

    /// resolve メソッドは Viterbi の結果のみを返すことを確認
    #[test]
    fn test_resolve_returns_viterbi_result() -> anyhow::Result<()> {
        // resolve は低レベル API として Viterbi の結果のみを返す
        let dict = HashMap::from([
            ("この".to_string(), vec!["この".to_string()]),
            ("も".to_string(), vec!["も".to_string()]),
            ("でる".to_string(), vec!["出る".to_string()]),
            ("もでる".to_string(), vec!["モデル".to_string()]),
        ]);

        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.add("この/この", 1.0);
        unigram_builder.add("も/も", 0.5);
        unigram_builder.add("出る/でる", 1.0);
        unigram_builder.add("モデル/もでる", 3.0);
        unigram_builder.set_total_words(1000);
        unigram_builder.set_unique_words(100);
        let system_unigram_lm = unigram_builder.build()?;

        let unigram_map = system_unigram_lm.as_hash_map();
        let kono_id = unigram_map.get("この/この").unwrap().0;
        let mo_id = unigram_map.get("も/も").unwrap().0;
        let deru_id = unigram_map.get("出る/でる").unwrap().0;

        let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
        bigram_builder.set_default_edge_cost(10.0);
        // Viterbi では「この→も→出る」の方がコストが低くなるように設定
        bigram_builder.add(kono_id, mo_id, 0.5);
        bigram_builder.add(mo_id, deru_id, 0.5);
        let system_bigram_lm = bigram_builder.build()?;

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let kana_trie = CedarwoodKanaTrie::build(vec![
            "この".to_string(),
            "も".to_string(),
            "でる".to_string(),
            "もでる".to_string(),
        ]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph_resolver = GraphResolver::default();

        let engine = BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            user_data: Arc::new(Mutex::new(UserData::default())),
            reranking_weights: ReRankingWeights::default(),
            skip_bigram_lm: None,
        };

        // to_lattice → resolve を呼び出し（リランキングなし）
        let lattice = engine.to_lattice("このもでる", None)?;
        let result = engine.resolve(&lattice)?;

        // Viterbi の結果（"この/も/出る"）が返されることを確認
        let text: Vec<String> = result
            .iter()
            .filter_map(|segment| segment.first().map(|c| c.surface.clone()))
            .collect();

        assert_eq!(text.join("/"), "この/も/出る");

        Ok(())
    }
}
