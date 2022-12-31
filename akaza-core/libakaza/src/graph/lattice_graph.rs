use std::collections::btree_map::BTreeMap;

use std::rc::Rc;

use crate::graph::word_node::WordNode;
use log::trace;

use crate::lm::system_unigram_lm::SystemUnigramLM;
use crate::user_side_data::user_data::UserData;

// 考えられる単語の列全てを含むようなグラフ構造
pub struct LatticeGraph {
    pub(crate) graph: BTreeMap<i32, Vec<WordNode>>,
    pub(crate) user_data: Rc<UserData>,
    pub(crate) system_unigram_lm: Rc<SystemUnigramLM>,
}

impl LatticeGraph {
    /// i文字目で終わるノードを探す
    pub(crate) fn node_list(&self, i: i32) -> Option<&Vec<WordNode>> {
        self.graph.get(&i)
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
    pub(crate) fn dump(&self) {
        // start 及び end は、byte 数単位
        for (end_pos, nodes) in self.graph.iter() {
            for node in nodes {
                print!("start={} end={}:", node.start_pos, end_pos);
                // 典型的には unicode で日本語文字が3バイトで2文字幅
                for _ in 0..(node.start_pos / 3 * 2) {
                    print!(" ");
                }
                println!("{}", node.kanji);
            }
        }
    }

    // for debugging purpose
    // graphviz の dot 形式で出力する。
    #[allow(unused)]
    pub fn dump_dot(&self) -> String {
        let mut buf = String::new();
        buf += "digraph Lattice {\n";
        // start 及び end は、byte 数単位
        for (end_pos, nodes) in self.graph.iter() {
            for node in nodes {
                buf += &*format!(
                    r#"    "{}/{}" [xlabel="{}"]{}"#,
                    node.kanji,
                    node.yomi,
                    self.get_node_cost(node),
                    "\n"
                );
                for prev_node in self.get_prev_nodes(node).unwrap() {
                    buf += &*format!(
                        r#"    "{}/{}" -> "{}/{}" [label="{}"]{}"#,
                        prev_node.kanji,
                        prev_node.yomi,
                        node.kanji,
                        node.yomi,
                        self.get_edge_cost(prev_node, node),
                        "\n"
                    );
                }
            }
        }
        buf += &*"}\n".to_string();
        buf
    }

    pub(crate) fn get_node_cost(&self, node: &WordNode) -> f32 {
        // 簡単のために、一旦スコアを文字列長とする。
        // 経験上、長い文字列のほうがあたり、というルールでもそこそこ変換できる。
        // TODO あとでちゃんと unigram のコストを使うよに変える。

        if let Some(user_cost) = self.user_data.get_unigram_cost(&node.kanji, &node.yomi) {
            // use user's score. if it's exists.
            return user_cost;
        }

        if node.kanji.len() < node.yomi.len() {
            // log10(1e-20)
            -20.0
        } else {
            // log10(1e-19)
            -19.0
        }

        /*
                if let Some(user_cost) = user_language_model.get_unigram_cost(&self.key) {
                    // use user's score, if it's exists.
                    return user_cost;
                }

                if self.system_word_id != UNKNOWN_WORD_ID {
                    self.total_cost = Some(self.system_unigram_cost);
                    return self.system_unigram_cost;
                } else {
                    // 労働者災害補償保険法 のように、システム辞書には採録されているが,
                    // 言語モデルには採録されていない場合,漢字候補を先頭に持ってくる。
                    return if self.word.len() < self.yomi.len() {
                        // 読みのほうが短いので、漢字。
                        ulm.get_default_cost_for_short()
                    } else {
                        ulm.get_default_cost()
                    };
                }
        */
    }

    pub(crate) fn get_edge_cost(&self, _prev: &WordNode, _node: &WordNode) -> f32 {
        // TODO: あとで実装する
        0.0
    }
}
