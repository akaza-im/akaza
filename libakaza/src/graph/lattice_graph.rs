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
        self.graph.get(&n)
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
        let user_data = self.user_data.lock().unwrap();
        self.get_node_cost_with_user_data(node, &user_data)
    }

    /// ロック済みの UserData を受け取るバージョン。
    /// resolve 内で一度だけロックを取り、ループ内ではこのメソッドを使うことで
    /// Mutex lock のオーバーヘッドを削減する。
    pub(crate) fn get_node_cost_with_user_data(
        &self,
        node: &WordNode,
        user_data: &UserData,
    ) -> f32 {
        if let Some(user_cost) = user_data.get_unigram_cost(node) {
            info!("Use user's node score: {:?}", node);
            // use user's score. if it's exists.
            return user_cost;
        }

        if let Some((_, system_unigram_cost)) = node.word_id_and_score {
            trace!("HIT!: {}, {}", node.key(), system_unigram_cost);
            system_unigram_cost
        } else if node.surface.len() < node.yomi.len() {
            // 労働者災害補償保険法 のように、システム辞書には wikipedia から採録されているが,
            // 言語モデルには採録されていない場合,漢字候補を先頭に持ってくる。
            // つまり、変換後のほうが短くなるもののほうをコストを安くしておく。
            self.system_unigram_lm.get_cost(1)
        } else {
            self.system_unigram_lm.get_cost(0)
        }
    }

    pub(crate) fn get_edge_cost(&self, prev: &WordNode, node: &WordNode) -> f32 {
        let user_data = self.user_data.lock().unwrap();
        self.get_edge_cost_with_user_data(prev, node, &user_data)
    }

    /// ロック済みの UserData を受け取るバージョン。
    /// resolve 内で一度だけロックを取り、ループ内ではこのメソッドを使うことで
    /// Mutex lock のオーバーヘッドを削減する。
    pub(crate) fn get_edge_cost_with_user_data(
        &self,
        prev: &WordNode,
        node: &WordNode,
        user_data: &UserData,
    ) -> f32 {
        if let Some(cost) = user_data.get_bigram_cost(prev, node) {
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

    /// エッジコストと既知 bigram かどうかを返す。
    /// リランキング用にコスト内訳を追跡するために使用する。
    pub(crate) fn get_edge_cost_detail_with_user_data(
        &self,
        prev: &WordNode,
        node: &WordNode,
        user_data: &UserData,
    ) -> (f32, bool) {
        if let Some(cost) = user_data.get_bigram_cost(prev, node) {
            return (cost, true);
        }

        let Some((prev_id, _)) = prev.word_id_and_score else {
            return (self.system_bigram_lm.get_default_edge_cost(), false);
        };
        let Some((node_id, _)) = node.word_id_and_score else {
            return (self.system_bigram_lm.get_default_edge_cost(), false);
        };
        if let Some(cost) = self.system_bigram_lm.get_edge_cost(prev_id, node_id) {
            (cost, true)
        } else {
            (self.system_bigram_lm.get_default_edge_cost(), false)
        }
    }

    pub fn get_default_edge_cost(&self) -> f32 {
        self.system_bigram_lm.get_default_edge_cost()
    }

    /// user_data のロックを取得する。
    /// resolve 時に一度だけロックを取り、ループ中は保持することで
    /// Mutex lock のオーバーヘッドを削減する。
    pub(crate) fn lock_user_data(&self) -> std::sync::MutexGuard<'_, UserData> {
        self.user_data.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::candidate::Candidate;
    use crate::graph::word_node::WordNode;
    use crate::lm::system_bigram::MarisaSystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;

    use super::*;

    fn setup_test_graph() -> anyhow::Result<
        LatticeGraph<
            crate::lm::system_unigram_lm::MarisaSystemUnigramLM,
            crate::lm::system_bigram::MarisaSystemBigramLM,
        >,
    > {
        // システムunigram言語モデルを構築
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.add("私/わたし", 1.5);
        unigram_builder.add("彼/かれ", 2.0);
        unigram_builder.set_total_words(100);
        unigram_builder.set_unique_words(50);
        let system_unigram_lm = unigram_builder.build()?;

        // システムbigram言語モデルを構築
        let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
        bigram_builder.set_default_edge_cost(10.0);
        let unigram_map = system_unigram_lm.as_hash_map();
        let watashi_id = unigram_map.get("私/わたし").unwrap().0;
        let kare_id = unigram_map.get("彼/かれ").unwrap().0;
        bigram_builder.add(watashi_id, kare_id, 0.5);
        let system_bigram_lm = bigram_builder.build()?;

        // グラフを構築
        let mut graph = BTreeMap::new();
        graph.insert(0, vec![WordNode::create_bos()]);

        // "わたし" のノード
        let watashi_node = WordNode::new(0, "私", "わたし", Some((watashi_id, 1.5)), false);
        graph.insert(9, vec![watashi_node.clone()]);

        // "かれ" のノード
        let kare_node = WordNode::new(9, "彼", "かれ", Some((kare_id, 2.0)), false);
        graph.insert(18, vec![kare_node.clone()]);

        // "ひらがな" のノード（言語モデルにない）
        let hiragana_node = WordNode::new(18, "ひらがな", "ひらがな", None, true);
        graph.insert(30, vec![hiragana_node]);

        graph.insert(31, vec![WordNode::create_eos(30)]);

        Ok(LatticeGraph {
            yomi: "わたしかれひらがな".to_string(),
            graph,
            user_data: Arc::new(Mutex::new(UserData::default())),
            system_unigram_lm: Rc::new(system_unigram_lm),
            system_bigram_lm: Rc::new(system_bigram_lm),
        })
    }

    #[test]
    fn test_get_node_cost_system_score() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let watashi_node = graph.node_list(9).unwrap().first().unwrap();

        let cost = graph.get_node_cost(watashi_node);
        assert_eq!(cost, 1.5); // システム辞書のスコア

        Ok(())
    }

    #[test]
    fn test_get_node_cost_unknown_word() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let hiragana_node = graph.node_list(30).unwrap().first().unwrap();

        let cost = graph.get_node_cost(hiragana_node);
        // 言語モデルにない単語はデフォルトコスト
        // ひらがなはsurface.len() >= yomi.len() なので get_cost(0)
        let expected_cost = graph.system_unigram_lm.get_cost(0);
        assert_eq!(cost, expected_cost);

        Ok(())
    }

    #[test]
    fn test_get_node_cost_user_score_priority() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let watashi_node = graph.node_list(9).unwrap().first().unwrap();

        // ユーザースコアを記録
        graph
            .user_data
            .lock()
            .unwrap()
            .record_entries(&[Candidate::new("わたし", "私", 0.1)]);

        let cost = graph.get_node_cost(watashi_node);
        // ユーザースコアが優先されることを確認（システムスコアより低いはず）
        assert!(cost < 1.5); // システムスコアは1.5

        Ok(())
    }

    #[test]
    fn test_get_edge_cost_system_bigram() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let watashi_node = graph.node_list(9).unwrap().first().unwrap();
        let kare_node = graph.node_list(18).unwrap().first().unwrap();

        let cost = graph.get_edge_cost(watashi_node, kare_node);
        // bigram言語モデルに登録されているスコア
        assert!(cost > 0.4 && cost < 0.6); // f16精度の誤差を考慮

        Ok(())
    }

    #[test]
    fn test_get_edge_cost_default() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let kare_node = graph.node_list(18).unwrap().first().unwrap();
        let hiragana_node = graph.node_list(30).unwrap().first().unwrap();

        let cost = graph.get_edge_cost(kare_node, hiragana_node);
        // 言語モデルに登録されていないエッジはデフォルトコスト
        assert_eq!(cost, 10.0);

        Ok(())
    }

    // TODO: ユーザーバイグラムスコアのテストを追加
    // BiGramUserStats の API を直接使用する必要がある

    #[test]
    fn test_get_prev_nodes() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;
        let kare_node = graph.node_list(18).unwrap().first().unwrap();

        let prev_nodes = graph.get_prev_nodes(kare_node).unwrap();
        assert_eq!(prev_nodes.len(), 1);
        assert_eq!(prev_nodes[0].surface, "私");

        Ok(())
    }

    #[test]
    fn test_node_list() -> anyhow::Result<()> {
        let graph = setup_test_graph()?;

        assert!(graph.node_list(0).is_some()); // BOS
        assert!(graph.node_list(9).is_some()); // わたし
        assert!(graph.node_list(18).is_some()); // かれ
        assert!(graph.node_list(30).is_some()); // ひらがな
        assert!(graph.node_list(31).is_some()); // EOS
        assert!(graph.node_list(100).is_none()); // 存在しない位置

        Ok(())
    }

    #[test]
    fn test_get_node_cost_kanji_shortening_bonus() -> anyhow::Result<()> {
        let _graph = setup_test_graph()?;

        // 変換後のほうが短くなる単語を追加（漢字変換）
        let mut graph_with_kanji = setup_test_graph()?;
        let kanji_node = WordNode::new(
            0,
            "労働者災害補償保険法",                   // 33バイト
            "ろうどうしゃさいがいほしょうほけんほう", // 63バイト
            None,                                     // 言語モデルにない
            false,
        );
        graph_with_kanji.graph.insert(63, vec![kanji_node.clone()]);

        let cost = graph_with_kanji.get_node_cost(&kanji_node);
        // surface.len() < yomi.len() なので get_cost(1)
        let expected_cost = graph_with_kanji.system_unigram_lm.get_cost(1);
        assert_eq!(cost, expected_cost);

        Ok(())
    }
}
