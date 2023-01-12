use std::collections::{HashMap, VecDeque};

use anyhow::Context;
use log::trace;

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;

#[derive(Debug)]
pub struct Candidate {
    pub kanji: String,
    pub yomi: String,
    pub cost: f32,
}

impl Candidate {
    pub fn new(yomi: &str, surface: &str, cost: f32) -> Candidate {
        Candidate {
            yomi: yomi.to_string(),
            kanji: surface.to_string(),
            cost,
        }
    }
}

/**
 * Segmenter により分割されたかな表現から、グラフを構築する。
 */
#[derive(Default)]
pub struct GraphResolver {}

impl GraphResolver {
    /**
     * ビタビアルゴリズムで最適な経路を見つける。
     */
    pub fn resolve(&self, lattice: &LatticeGraph) -> anyhow::Result<Vec<VecDeque<Candidate>>> {
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
                        node.kanji, node.start_pos, lattice
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
            if node.kanji != "__EOS__" {
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
                        kanji: f.kanji.clone(),
                        yomi: f.yomi.clone(),
                        cost: *costmap.get(f).unwrap(),
                    })
                    .collect();
                candidates
                    .make_contiguous()
                    .sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());
                candidates.push_front(Candidate {
                    kanji: node.kanji.clone(),
                    yomi: node.yomi.clone(),
                    cost: *costmap.get(node).unwrap(),
                });
                result.push(candidates);
            }
            node = prevmap
                .get(node)
                .unwrap_or_else(|| panic!("Cannot get previous node: {}", node.kanji));
        }
        result.reverse();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Write;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use anyhow::Result;

    use crate::graph::graph_builder::GraphBuilder;
    use crate::graph::segmenter::{SegmentationResult, Segmenter};
    use crate::kana_kanji_dict::{KanaKanjiDict, KanaKanjiDictBuilder};
    use crate::kana_trie::marisa_kana_trie::MarisaKanaTrie;
    use crate::lm::system_bigram::SystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::SystemUnigramLMBuilder;
    use crate::user_side_data::user_data::UserData;

    use super::*;

    #[test]
    fn test_resolver() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = MarisaKanaTrie::build(Vec::from([
            "abc".to_string(),
            "ab".to_string(),
            "c".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Box::new(kana_trie)]);
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
        let dict_builder = KanaKanjiDictBuilder::default();
        let dict = dict_builder.build();
        let system_unigram_lm_builder = SystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder.build();
        let system_bigram_lm_builder = SystemBigramLMBuilder::default();
        let system_bigram_lm = system_bigram_lm_builder.build();
        let user_data = UserData::default();
        let graph_builder = GraphBuilder::new_with_default_score(
            dict,
            Default::default(),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("abc", graph);
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "abc");
        Ok(())
    }

    #[test]
    fn test_kana_kanji() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = MarisaKanaTrie::build(Vec::from([
            "わたし".to_string(),
            "わた".to_string(),
            "し".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Box::new(kana_trie)]);
        let graph = graph_builder.build("わたし", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ]))
        );

        let mut dict_builder = KanaKanjiDictBuilder::default();
        dict_builder.add("わたし", "私/渡し");

        let yomi = "わたし".to_string();

        let dict = dict_builder.build();
        let system_unigram_lm_builder = SystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder.build();
        let system_bigram_lm_builder = SystemBigramLMBuilder::default();
        let system_bigram_lm = system_bigram_lm_builder.build();
        let mut user_data = UserData::default();
        // 私/わたし のスコアをガッと上げる。
        user_data.record_entries(&["私/わたし".to_string()]);
        let graph_builder = GraphBuilder::new_with_default_score(
            dict,
            KanaKanjiDict::default(),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct(&yomi, graph);
        // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
        // File::create("/tmp/lattice.dot")
        //     .unwrap()
        //     .write_all(lattice.dump_cost_dot().as_bytes())
        //     .unwrap();
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "私");
        Ok(())
    }
}
