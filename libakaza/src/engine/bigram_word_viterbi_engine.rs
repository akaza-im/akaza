use std::collections::VecDeque;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::engine::base::HenkanEngine;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::{Candidate, GraphResolver};
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji_dict::KanaKanjiDict;
use crate::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use crate::lm::system_bigram::MarisaSystemBigramLM;
use crate::lm::system_unigram_lm::MarisaSystemUnigramLM;
use crate::romkan::RomKanConverter;
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
pub struct BigramWordViterbiEngine {
    graph_builder: GraphBuilder,
    pub segmenter: Segmenter,
    pub graph_resolver: GraphResolver,
    romkan_converter: RomKanConverter,
    pub user_data: Arc<Mutex<UserData>>,
}

impl BigramWordViterbiEngine {}

impl HenkanEngine for BigramWordViterbiEngine {
    fn learn(&mut self, surface_kanas: &[String]) {
        self.user_data.lock().unwrap().record_entries(surface_kanas);
    }

    fn resolve(&self, lattice: &LatticeGraph) -> Result<Vec<VecDeque<Candidate>>> {
        self.graph_resolver.resolve(lattice)
    }

    fn to_lattice(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> Result<LatticeGraph> {
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
}

impl BigramWordViterbiEngineBuilder {
    pub fn new(system_data_dir: &str) -> BigramWordViterbiEngineBuilder {
        BigramWordViterbiEngineBuilder {
            system_data_dir: system_data_dir.to_string(),
            user_data: None,
        }
    }

    pub fn user_data(&mut self, user_data: Arc<Mutex<UserData>>) -> &mut Self {
        self.user_data = Some(user_data);
        self
    }

    pub fn build(&self) -> Result<BigramWordViterbiEngine> {
        let system_data_loader = SystemDataLoader::load(self.system_data_dir.as_str())?;

        let segmenter = Segmenter::new(vec![Box::new(system_data_loader.system_kana_trie)]);

        let user_data = if let Some(d) = &self.user_data {
            d.clone()
        } else {
            Arc::new(Mutex::new(UserData::default()))
        };

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
}
