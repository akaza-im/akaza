use std::rc::Rc;

use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::GraphResolver;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji_dict::{KanaKanjiDict, KanaKanjiDictBuilder};
use crate::kana_trie::KanaTrieBuilder;
use anyhow::Result;

use crate::lm::system_bigram::{SystemBigramLM, SystemBigramLMBuilder};
use crate::lm::system_unigram_lm::{SystemUnigramLM, SystemUnigramLMBuilder};
use crate::user_side_data::user_data::UserData;

pub struct Akaza {
    graph_builder: GraphBuilder,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
}

impl Akaza {
    pub fn convert_to_string(&self, yomi: &str) -> Result<String> {
        let segmentation_result = self.segmenter.build(yomi);
        let lattice = self.graph_builder.construct(yomi, segmentation_result);
        self.graph_resolver.viterbi(yomi, lattice)
    }
}

#[derive(Default)]
pub struct AkazaBuilder {
    system_data_dir: Option<String>,
}

impl AkazaBuilder {
    pub fn system_data_dir(&mut self, system_data_dir: &str) -> &mut AkazaBuilder {
        self.system_data_dir = Some(system_data_dir.to_string());
        self
    }

    pub fn build(&self) -> Result<Akaza> {
        let system_unigram_lm = match &self.system_data_dir {
            Some(dir) => {
                let path = dir.to_string() + "/lm_v2_1gram.trie";
                SystemUnigramLM::load(path.as_str())?
            }
            None => SystemUnigramLMBuilder::default().build(),
        };
        let system_bigram_lm = match &self.system_data_dir {
            Some(dir) => {
                let path = dir.to_string() + "/lm_v2_2gram.trie";
                SystemBigramLM::load(path.as_str())?
            }
            None => SystemBigramLMBuilder::default().build(),
        };

        let system_kana_kanji_dict = match &self.system_data_dir {
            Some(dir) => {
                let path = dir.to_string() + "/system_dict.trie";
                KanaKanjiDict::load(path.as_str())?
            }
            None => KanaKanjiDictBuilder::default().build(),
        };

        // TODO キャッシュする余地
        let mut system_dict_yomis_builder = KanaTrieBuilder::default();
        for yomi in system_kana_kanji_dict.all_yomis().unwrap() {
            system_dict_yomis_builder.add(&yomi);
        }
        let system_kana_trie = system_dict_yomis_builder.build();

        let segmenter = Segmenter::new(vec![system_kana_trie]);

        // TODO use real user data.
        let user_data = UserData::default();

        let graph_builder = GraphBuilder::new(
            system_kana_kanji_dict,
            Rc::new(user_data),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let graph_resolver = GraphResolver::default();

        Ok(Akaza {
            graph_builder: graph_builder,
            segmenter: segmenter,
            graph_resolver: graph_resolver,
        })
    }
}
