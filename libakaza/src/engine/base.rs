use crate::graph::graph_resolver::Candidate;
use crate::graph::lattice_graph::LatticeGraph;
use std::collections::vec_deque::VecDeque;
use std::ops::Range;

pub trait HenkanEngine {
    fn learn(&mut self, surface_kanas: &[String]);

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<VecDeque<Candidate>>> {
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

    fn resolve(&self, lattice: &LatticeGraph) -> anyhow::Result<Vec<VecDeque<Candidate>>>;

    fn to_lattice(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<LatticeGraph>;
}
