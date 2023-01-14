use std::collections::vec_deque::VecDeque;
use std::ops::Range;

use crate::graph::graph_resolver::Candidate;

pub trait HenkanEngine {
    fn learn(&mut self, candidates: &[Candidate]);

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<VecDeque<Candidate>>>;
}
