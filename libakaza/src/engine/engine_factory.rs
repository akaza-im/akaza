use crate::config::{DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};
use crate::dict::loader::{load_dicts, load_dicts_with_cache};
use crate::engine::base::HenkanEngine;
use crate::engine::bigram_word_viterbi_engine::BigramWordViterbiEngine;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::GraphResolver;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::user_side_data::user_data::UserData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct EngineFactory {
    user_data: Option<Arc<Mutex<UserData>>>,
    config: EngineConfig,
}

impl EngineFactory {
    pub fn new(config: EngineConfig) -> EngineFactory {
        EngineFactory {
            user_data: None,
            config,
        }
    }

    pub fn user_data(&mut self, user_data: Arc<Mutex<UserData>>) -> &mut Self {
        self.user_data = Some(user_data);
        self
    }

    pub fn build(&self) -> anyhow::Result<Box<dyn HenkanEngine>> {
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

        Ok(Box::new(BigramWordViterbiEngine::new(
            graph_builder,
            segmenter,
            graph_resolver,
            user_data,
        )))
    }

    fn try_load(model_dir: &str, name: &str) -> anyhow::Result<String> {
        Ok(model_dir.to_string() + "/" + name)
    }
}
