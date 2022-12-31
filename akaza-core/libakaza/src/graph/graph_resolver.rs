use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;
use std::collections::HashMap;

// 次に必要なのは、分割された文字列から、グラフを構築する仕組みである。
pub struct GraphResolver {}

impl GraphResolver {
    pub fn new() -> GraphResolver {
        GraphResolver {}
    }

    pub fn viterbi(&self, yomi: &String, lattice: LatticeGraph) -> String {
        let mut prevmap: HashMap<&WordNode, &WordNode> = HashMap::new();
        let mut costmap: HashMap<&WordNode, f32> = HashMap::new();

        lattice.dump();
        lattice.dump_dot();

        for i in 1..yomi.len() + 2 {
            let Some(nodes) = lattice.node_list(i as i32) else {
                continue;
            };
            for node in nodes {
                let node_cost = lattice.get_node_cost(node);
                println!("kanji={}, Cost={}", node, node_cost);
                let mut cost = f32::MIN;
                let mut shortest_prev = None;
                let prev_nodes = lattice.get_prev_nodes(node).unwrap_or_else(|| {
                    panic!(
                        "Cannot get prev nodes for '{}' start={}",
                        node.kanji, node.start_pos
                    )
                });
                for prev in prev_nodes {
                    let edge_cost = lattice.get_edge_cost(prev, node);
                    let prev_cost = costmap.get(prev).unwrap_or(&0_f32); // unwrap が必要なのは、 __BOS__ 用。
                    let tmp_cost = prev_cost + edge_cost + node_cost;
                    println!(
                        "Replace??? prev_cost={} tmp_cost={} < cost={}: {}",
                        prev_cost, tmp_cost, cost, prev
                    );
                    // コストが最大な経路を選ぶようにする。
                    // そういうふうにコストを付与しているので。
                    if cost < tmp_cost {
                        if shortest_prev.is_none() {
                            println!("Replace None by {}", prev);
                        } else {
                            println!("Replace {} by {}", shortest_prev.unwrap(), prev);
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
        let mut result: Vec<String> = Vec::new();
        while node != bos {
            if node.kanji != "__EOS__" {
                result.push(node.kanji.to_string());
            }
            node = prevmap
                .get(node)
                .unwrap_or_else(|| panic!("Cannot get previous node: {}", node.kanji));
        }
        result.reverse();
        result.join("")
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::rc::Rc;

    use crate::graph::graph_builder::GraphBuilder;
    use crate::graph::segmenter::Segmenter;
    use tempfile::NamedTempFile;

    use crate::kana_kanji_dict::KanaKanjiDictBuilder;
    use crate::kana_trie::KanaTrieBuilder;
    use crate::lm::system_unigram_lm::SystemUnigramLMBuilder;
    use crate::user_side_data::user_data::UserData;

    use super::*;

    #[test]
    fn test() {
        let mut builder = KanaTrieBuilder::default();
        builder.add(&"わたし".to_string());
        builder.add(&"わた".to_string());
        builder.add(&"し".to_string());
        let kana_trie = builder.build();

        let graph_builder = Segmenter::new(vec![kana_trie]);
        let graph = graph_builder.build(&"わたし".to_string());
        assert_eq!(
            graph,
            HashMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ])
        )
    }

    #[test]
    fn test_resolver() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut builder = KanaTrieBuilder::default();
        builder.add(&"abc".to_string());
        builder.add(&"ab".to_string());
        builder.add(&"c".to_string());
        let kana_trie = builder.build();

        let graph_builder = Segmenter::new(vec![kana_trie]);
        let graph = graph_builder.build(&"abc".to_string());
        assert_eq!(
            graph,
            HashMap::from([
                (2, vec!["ab".to_string()]),
                (3, vec!["abc".to_string(), "c".to_string()]),
            ])
        );

        // -1  0  1  2
        // BOS a  b  c
        let dict_builder = KanaKanjiDictBuilder::default();
        let dict = dict_builder.build();
        let system_unigram_lm_builder = SystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder.build();
        let user_data = UserData::load(
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
        );
        let graph_builder = GraphBuilder::new(dict, Rc::new(user_data), Rc::new(system_unigram_lm));
        let lattice = graph_builder.construct(&"abc".to_string(), graph);
        let resolver = GraphResolver::new();
        let result = resolver.viterbi(&"abc".to_string(), lattice);
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_kana_kanji() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut builder = KanaTrieBuilder::default();
        builder.add(&"わたし".to_string());
        builder.add(&"わた".to_string());
        builder.add(&"し".to_string());
        let kana_trie = builder.build();

        let graph_builder = Segmenter::new(vec![kana_trie]);
        let graph = graph_builder.build(&"わたし".to_string());
        assert_eq!(
            graph,
            HashMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ])
        );

        let mut dict_builder = KanaKanjiDictBuilder::default();
        dict_builder.add("わたし", "私/渡し");

        let yomi = "わたし".to_string();

        // TODO このへん、ちょっとコピペしまくらないといけなくて渋い。
        let dict = dict_builder.build();
        let system_unigram_lm_builder = SystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder.build();
        let mut user_data = UserData::load(
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            &NamedTempFile::new()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
        );
        // 私/わたし のスコアをガッと上げる。
        user_data.record_entries(vec!["私/わたし".to_string()]);
        let graph_builder = GraphBuilder::new(dict, Rc::new(user_data), Rc::new(system_unigram_lm));
        let lattice = graph_builder.construct(&yomi, graph);
        // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
        File::create("/tmp/lattice.dot")
            .unwrap()
            .write_all(lattice.dump_dot().as_bytes())
            .unwrap();
        let resolver = GraphResolver::new();
        let result = resolver.viterbi(&yomi, lattice);
        assert_eq!(result, "私");
    }
}
