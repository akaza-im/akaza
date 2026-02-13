use std::ops::Range;

use crate::graph::candidate::Candidate;
use crate::graph::graph_resolver::KBestPath;

pub trait HenkanEngine {
    fn learn(&mut self, candidates: &[Candidate]);

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<Vec<Candidate>>>;

    /// k-best ビタビで上位 k 個の分節パターンを返す。
    /// 各パスは分節パターン（文節×漢字候補）と真のパスコストを持つ。
    fn convert_k_best(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
        k: usize,
    ) -> anyhow::Result<Vec<KBestPath>>;
}
