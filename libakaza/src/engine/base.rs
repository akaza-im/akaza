use std::ops::Range;

use crate::graph::candidate::Candidate;

pub trait HenkanEngine {
    fn learn(&mut self, candidates: &[Candidate]);

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<Vec<Candidate>>>;
}
