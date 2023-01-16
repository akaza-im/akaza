use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;

use crate::graph::candidate::Candidate;
use anyhow::Context;

use log::trace;

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};

/**
 * Segmenter により分割されたかな表現から、グラフを構築する。
 */
#[derive(Default)]
pub struct GraphResolver {}

impl GraphResolver {
    /**
     * ビタビアルゴリズムで最適な経路を見つける。
     */
    pub fn resolve<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        lattice: &LatticeGraph<U, B>,
    ) -> anyhow::Result<Vec<VecDeque<Candidate>>> {
        let yomi = &lattice.yomi;
        let mut prevmap: HashMap<&WordNode, &WordNode> = HashMap::new();
        let mut costmap: HashMap<&WordNode, f32> = HashMap::new();

        for i in 1..yomi.len() + 2 {
            let Some(nodes) = &lattice.node_list(i as i32) else {
                continue;
            };
            for node in *nodes {
                let node_cost = lattice.get_node_cost(node);
                trace!("kanji={}, Cost={}", node, node_cost);
                let mut cost = f32::MAX;
                let mut shortest_prev = None;
                let prev_nodes = lattice.get_prev_nodes(node).with_context(|| {
                    format!(
                        "Cannot get prev nodes for '{}' start={} lattice={:?}",
                        node.surface, node.start_pos, lattice
                    )
                })?;
                for prev in prev_nodes {
                    let edge_cost = lattice.get_edge_cost(prev, node);
                    let prev_cost = costmap.get(prev).unwrap_or(&0_f32); // unwrap が必要なのは、 __BOS__ 用。
                    let tmp_cost = prev_cost + edge_cost + node_cost;
                    trace!(
                        "Replace??? prev_cost={} tmp_cost={} < cost={}: {}",
                        prev_cost,
                        tmp_cost,
                        cost,
                        prev
                    );
                    // コストが最小な経路を選ぶようにする。
                    // そういうふうにコストを付与しているので。
                    if cost > tmp_cost {
                        if shortest_prev.is_none() {
                            trace!("Replace None by {}", prev);
                        } else {
                            trace!("Replace {} by {}", shortest_prev.unwrap(), prev);
                        }
                        cost = tmp_cost;
                        shortest_prev = Some(prev);
                    }
                }
                prevmap.insert(node, shortest_prev.unwrap());
                costmap.insert(node, cost);
            }
        }

        let eos = lattice
            .get((yomi.len() + 1) as i32)
            .unwrap()
            .get(0)
            .unwrap();
        let bos = lattice.get(0).unwrap().get(0).unwrap();
        let mut node = eos;
        let mut result: Vec<VecDeque<Candidate>> = Vec::new();
        while node != bos {
            if node.surface != "__EOS__" {
                // 同一の開始位置、終了位置を持つものを集める。
                let end_pos = node.start_pos + (node.yomi.len() as i32);
                let mut candidates: VecDeque<Candidate> = lattice
                    .node_list(end_pos)
                    .unwrap()
                    .iter()
                    .filter(|alt_node| {
                        alt_node.start_pos == node.start_pos
                            && alt_node.yomi.len() == node.yomi.len()
                            && alt_node != &node
                    })
                    .map(|f| Candidate {
                        surface: f.surface.clone(),
                        yomi: f.yomi.clone(),
                        cost: *costmap.get(f).unwrap(),
                    })
                    .collect();
                candidates
                    .make_contiguous()
                    .sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());
                candidates.push_front(Candidate {
                    surface: node.surface.clone(),
                    yomi: node.yomi.clone(),
                    cost: *costmap.get(node).unwrap(),
                });
                result.push(candidates);
            }
            node = prevmap
                .get(node)
                .unwrap_or_else(|| panic!("Cannot get previous node: {}", node.surface));
        }
        result.reverse();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::btree_map::BTreeMap;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use anyhow::Result;

    use crate::graph::graph_builder::GraphBuilder;
    use crate::graph::segmenter::{SegmentationResult, Segmenter};
    use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
    use crate::lm::system_bigram::MarisaSystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;
    use crate::user_side_data::user_data::UserData;

    use super::*;

    #[test]
    fn test_resolver() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "abc".to_string(),
            "ab".to_string(),
            "c".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = graph_builder.build("abc", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (2, vec!["ab".to_string()]),
                (3, vec!["abc".to_string(), "c".to_string()]),
            ]))
        );

        // -1  0  1  2
        // BOS a  b  c
        let system_unigram_lm = MarisaSystemUnigramLMBuilder::default()
            .set_default_cost(20_f32)
            .set_default_cost_for_short(19_f32)
            .build();
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
            .set_default_edge_cost(20_f32)
            .build()?;
        let user_data = UserData::default();
        let graph_builder = GraphBuilder::new_with_default_score(
            HashMap::new(),
            Default::default(),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("abc", &graph);
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "abc");
        Ok(())
    }

    #[test]
    fn test_kana_kanji() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "わたし".to_string(),
            "わた".to_string(),
            "し".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = graph_builder.build("わたし", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ]))
        );

        let dict = HashMap::from([(
            "わたし".to_string(),
            vec!["私".to_string(), "渡し".to_string()],
        )]);

        let yomi = "わたし".to_string();

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder
            .set_default_cost(19_f32)
            .set_default_cost_for_short(20_f32)
            .build();
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
            .set_default_edge_cost(20_f32)
            .build()?;
        let mut user_data = UserData::default();
        // 私/わたし のスコアをガッと上げる。
        user_data.record_entries(&[Candidate::new("わたし", "私", 0_f32)]);
        let graph_builder = GraphBuilder::new_with_default_score(
            dict,
            HashMap::new(),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct(&yomi, &graph);
        // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
        // File::create("/tmp/lattice.dot")
        //     .unwrap()
        //     .write_all(lattice.dump_cost_dot().as_bytes())
        //     .unwrap();
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "私");
        Ok(())
    }
}
