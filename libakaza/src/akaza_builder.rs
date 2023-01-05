use std::collections::VecDeque;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::{Candidate, GraphResolver};
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji_dict::{KanaKanjiDict, KanaKanjiDictBuilder};
use crate::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use crate::lm::system_bigram::{SystemBigramLM, SystemBigramLMBuilder};
use crate::lm::system_unigram_lm::{SystemUnigramLM, SystemUnigramLMBuilder};
use crate::romkan::RomKanConverter;
use crate::user_side_data::user_data::UserData;

pub struct Akaza {
    graph_builder: GraphBuilder,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    romkan_converter: RomKanConverter,
    pub user_data: Arc<Mutex<UserData>>,
}

impl Akaza {
    pub fn learn(&mut self, surface_kanas: &Vec<String>) {
        self.user_data.lock().unwrap().record_entries(surface_kanas);
    }

    pub fn convert(
        &self,
        yomi: &str,
        force_ranges: &Vec<Range<usize>>,
    ) -> Result<Vec<VecDeque<Candidate>>> {
        // 先頭が大文字なケースと、URL っぽい文字列のときは変換処理を実施しない。
        if (!yomi.is_empty()
            && yomi.chars().next().unwrap().is_ascii_uppercase()
            && force_ranges.is_empty())
            || yomi.starts_with("https://")
            || yomi.starts_with("http://")
        {
            return Ok(vec![VecDeque::from([Candidate::new(yomi, yomi, 0_f32)])]);
        }

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
        self.graph_resolver.resolve(&lattice)
    }
}

#[derive(Default)]
pub struct AkazaBuilder {
    system_data_dir: Option<String>,
    user_data: Option<Arc<Mutex<UserData>>>,
}

impl AkazaBuilder {
    pub fn user_data(&mut self, user_data: Arc<Mutex<UserData>>) -> &mut Self {
        self.user_data = Some(user_data);
        self
    }

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
        let system_single_term_dict = match &self.system_data_dir {
            Some(dir) => {
                let path = dir.to_string() + "/single_term.trie";
                KanaKanjiDict::load(path.as_str())?
            }
            None => KanaKanjiDictBuilder::default().build(),
        };

        // TODO 事前に静的生成可能。
        let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
        let system_kana_trie = MarisaKanaTrie::build(all_yomis);

        let segmenter = Segmenter::new(vec![Box::new(system_kana_trie)]);

        let user_data = if let Some(d) = &self.user_data {
            d.clone()
        } else {
            Arc::new(Mutex::new(UserData::default()))
        };

        let graph_builder = GraphBuilder::new(
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data.clone(),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );

        let graph_resolver = GraphResolver::default();

        let romkan_converter = RomKanConverter::new();

        Ok(Akaza {
            graph_builder,
            segmenter,
            graph_resolver,
            romkan_converter,
            user_data,
        })
    }
}
