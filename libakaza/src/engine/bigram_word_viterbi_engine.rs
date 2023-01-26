use std::ops::Range;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::engine::base::HenkanEngine;
use crate::graph::candidate::Candidate;
use crate::graph::graph_builder::GraphBuilder;
use crate::graph::graph_resolver::GraphResolver;
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::Segmenter;
use crate::kana_kanji::base::KanaKanjiDict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
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

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> BigramWordViterbiEngine<U, B, KD> {
    pub(crate) fn new(
        graph_builder: GraphBuilder<U, B, KD>,
        segmenter: Segmenter,
        graph_resolver: GraphResolver,
        user_data: Arc<Mutex<UserData>>,
    ) -> Self {
        BigramWordViterbiEngine {
            graph_builder,
            segmenter,
            graph_resolver,
            user_data,
        }
    }

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
