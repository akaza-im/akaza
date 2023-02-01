use std::collections::btree_map::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use log::{error, info, trace};

use crate::graph::word_node::WordNode;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::user_side_data::user_data::UserData;

// 考えられる単語の列全てを含むようなグラフ構造
pub struct LatticeGraph<U: SystemUnigramLM, B: SystemBigramLM> {
    pub(crate) yomi: String,
    pub(crate) graph: BTreeMap<i32, Vec<WordNode>>,
    pub(crate) user_data: Arc<Mutex<UserData>>,
    pub(crate) system_unigram_lm: Rc<U>,
    pub(crate) system_bigram_lm: Rc<B>,
}

impl<U: SystemUnigramLM, B: SystemBigramLM> Debug for LatticeGraph<U, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LatticeGraph(yomi={}, graph={:?})",
            self.yomi, self.graph
        )
    }
}

impl<U: SystemUnigramLM, B: SystemBigramLM> LatticeGraph<U, B> {
    /// i文字目で終わるノードを探す
    pub fn node_list(&self, end_pos: i32) -> Option<&Vec<WordNode>> {
        self.graph.get(&end_pos)
    }

    // -1  0  1 2
    // BOS わ た し
    //     [  ][ ]
    //     [     ]
    pub(crate) fn get_prev_nodes(&self, node: &WordNode) -> Option<&Vec<WordNode>> {
        // ここの処理を簡単にするために BOS が入っている、のだとおもう。
        trace!("get_prev_nodes: {}", node.start_pos - 1);
        self.graph.get(&(node.start_pos))
    }

    pub(crate) fn get(&self, n: i32) -> Option<&Vec<WordNode>> {
        return self.graph.get(&n);
    }

    // for debugging purpose
    #[allow(unused)]
    pub fn dump_position_dot(&self) -> String {
        let mut buf = String::new();
        buf += "digraph Lattice {\n";
        // start 及び end は、byte 数単位
        for (end_pos, nodes) in self.graph.iter() {
            for node in nodes {
                buf += &*format!(
                    r#"    {} -> "{}/{}"{}"#,
                    node.start_pos, node.surface, node.yomi, "\n"
                );
                buf += &*format!(
                    r#"    "{}/{}" -> {}{}"#,
                    node.surface, node.yomi, end_pos, "\n"
                );
            }
        }
        buf += &*"}\n".to_string();
        buf
    }

    fn is_match(s: &str, expected: &str) -> bool {
        if expected.contains(s) {
            return true;
        }
        false
    }

    // for debugging purpose
    /// コストが各ノードおよびエッジについているかを出力する。
    /// graphviz の dot 形式で出力する。
    #[allow(unused)]
    pub fn dump_cost_dot(&self, expected: &str) -> String {
        let mut buf = String::new();
        buf += "digraph Lattice {\n";

        // start 及び end は、byte 数単位
        for (end_pos, nodes) in self.graph.iter() {
            for node in nodes {
                if Self::is_match(node.surface.as_str(), expected) {
                    buf += &*format!(
                        r#"    "{}/{}" [xlabel="{}"]{}"#,
                        node.surface,
                        node.yomi,
                        self.get_node_cost(node),
                        "\n"
                    );
                    if let Some(prev_nodes) = self.get_prev_nodes(node) {
                        for prev_node in prev_nodes {
                            if Self::is_match(prev_node.surface.as_str(), expected) {
                                buf += &*format!(
                                    r#"    "{}/{}" -> "{}/{}" [label="{}"]{}"#,
                                    prev_node.surface,
                                    prev_node.yomi,
                                    node.surface,
                                    node.yomi,
                                    self.get_edge_cost(prev_node, node),
                                    "\n"
                                );
                            }
                        }
                    } else {
                        error!("Missing previous nodes for {}", node);
                    }
                }
            }
        }
        buf += &*"}\n".to_string();
        buf
    }

    pub(crate) fn get_node_cost(&self, node: &WordNode) -> f32 {
        if let Some(user_cost) = self.user_data.lock().unwrap().get_unigram_cost(node) {
            info!("Use user's node score: {:?}", node);
            // use user's score. if it's exists.
            return user_cost;
        }

        return if let Some((_, system_unigram_cost)) = node.word_id_and_score {
            trace!("HIT!: {}, {}", node.key(), system_unigram_cost);
            system_unigram_cost
        } else if node.surface == node.yomi {
            // 「ひらがな」で、変換しないエントリーを優先する。
            // 漢字で表現したい場合は、それを自分で変換して選択すればよろしい。
            // このようにしないと、例えば新語辞書に変なものが入ってきたときに
            // 優先されすぎてしまう。
            //
            // 新語には、短いものもあり、誤爆しがち。
            self.system_unigram_lm.get_cost(1)
        } else {
            self.system_unigram_lm.get_cost(0)
        };
    }

    pub(crate) fn get_edge_cost(&self, prev: &WordNode, node: &WordNode) -> f32 {
        if let Some(cost) = self.user_data.lock().unwrap().get_bigram_cost(prev, node) {
            return cost;
        }

        let Some((prev_id, _)) = prev.word_id_and_score else {
            return self.system_bigram_lm.get_default_edge_cost();
        };
        let Some((node_id, _)) = node.word_id_and_score else {
            return self.system_bigram_lm.get_default_edge_cost();
        };
        if let Some(cost) = self.system_bigram_lm.get_edge_cost(prev_id, node_id) {
            cost
        } else {
            self.system_bigram_lm.get_default_edge_cost()
        }
    }

    pub fn get_default_edge_cost(&self) -> f32 {
        self.system_bigram_lm.get_default_edge_cost()
    }
}
