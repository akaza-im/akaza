use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use log::trace;

use crate::kana_kanji_dict::KanaKanjiDict;
use crate::kana_trie::KanaTrie;

#[derive(PartialEq)]
struct WordNode {
    start_pos: i32,
    /// 漢字
    kanji: String,
    /// 読み仮名
    yomi: String,
    cost: f32,
}

impl Hash for WordNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_pos.hash(state);
        self.kanji.hash(state);
        u32::from_le_bytes(self.cost.to_le_bytes()).hash(state);
    }
}

impl Eq for WordNode {}

impl WordNode {
    fn create_bos() -> WordNode {
        WordNode {
            start_pos: 0,
            kanji: "__BOS__".to_string(),
            yomi: "__BOS__".to_string(),
            cost: 0_f32,
        }
    }
    fn create_eos(start_pos: i32) -> WordNode {
        WordNode {
            start_pos,
            kanji: "__EOS__".to_string(),
            yomi: "__EOS__".to_string(),
            cost: 0_f32,
        }
    }
    fn new(start_pos: i32, kanji: &str, yomi: &str) -> WordNode {
        WordNode {
            start_pos,
            kanji: kanji.to_string(),
            yomi: yomi.to_string(),
            cost: 0_f32,
        }
    }
}

impl Display for WordNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.kanji, self.yomi)
    }
}

struct Segmenter {
    tries: Vec<KanaTrie>,
}

impl Segmenter {
    pub(crate) fn new(tries: Vec<KanaTrie>) -> Segmenter {
        Segmenter { tries }
    }

    /**
     * 読みをうけとって、グラフを構築する。
     */
    // シフトを押して → を押したときのような処理の場合、
    // このメソッドに入ってくる前に別に処理する前提。
    fn build(&self, yomi: &String) -> HashMap<usize, Vec<String>> {
        let mut queue: Vec<usize> = Vec::new(); // 検索対象となる開始位置
        queue.push(0);
        let mut seen: HashSet<usize> = HashSet::new();

        // 終了位置ごとの候補単語リスト
        let mut words_ends_at: HashMap<usize, Vec<String>> = HashMap::new();

        while queue.len() > 0 {
            let start_pos = queue.pop().unwrap();
            if seen.contains(&start_pos) {
                continue;
            } else {
                seen.insert(start_pos);
            }

            let yomi = &yomi[start_pos..];
            if yomi.is_empty() {
                continue;
            }

            let mut candidates: HashSet<String> = HashSet::new();
            for trie in &self.tries {
                let got = trie.common_prefix_search(&yomi.to_string());
                for g in got {
                    candidates.insert(g);
                }
            }
            if candidates.len() > 0 {
                for candidate in &candidates {
                    let ends_at = start_pos + candidate.len();

                    let vec = words_ends_at.entry(ends_at).or_insert(Vec::new());
                    vec.push(candidate.clone());

                    queue.push(start_pos + candidate.len());
                }
            } else {
                // 辞書に1文字も候補がない場合は先頭文字を取り出してグラフに入れる
                // ここは改善の余地がありそう。

                let (i, _) = yomi.char_indices().nth(1).unwrap();
                let first = &yomi[0..i];
                let ends_at = start_pos + first.len();

                let vec = words_ends_at.entry(ends_at).or_insert(Vec::new());
                vec.push(first.to_string());

                queue.push(start_pos + first.len())
            }
        }

        return words_ends_at;
    }
}

struct GraphBuilder {
    system_kana_kanji_dict: KanaKanjiDict,
}
impl GraphBuilder {
    fn new(system_kana_kanji_dict: KanaKanjiDict) -> GraphBuilder {
        GraphBuilder {
            system_kana_kanji_dict,
        }
    }

    fn construct(&self, yomi: &String, words_ends_at: HashMap<usize, Vec<String>>) -> LatticeGraph {
        // このグラフのインデクスは単語の終了位置。
        let mut graph: BTreeMap<i32, Vec<WordNode>> = BTreeMap::new();
        graph.insert(0, vec![WordNode::create_bos()]);
        graph.insert(
            (yomi.len() + 1) as i32,
            vec![WordNode::create_eos(yomi.len() as i32)],
        );

        for (end_pos, yomis) in words_ends_at {
            for yomi in yomis {
                let vec = graph.entry(end_pos as i32).or_insert(Vec::new());

                // ひらがなそのものもエントリーとして登録しておく。
                let node = WordNode::new((end_pos - yomi.len()) as i32, &yomi, &yomi);
                vec.push(node);

                // 漢字に変換した結果もあれば insert する。
                if let Some(kanjis) = self.system_kana_kanji_dict.find(&yomi) {
                    for kanji in kanjis {
                        let node = WordNode::new((end_pos - yomi.len()) as i32, &kanji, &yomi);
                        vec.push(node);
                    }
                }
            }
        }
        LatticeGraph { graph }
    }
}

// 考えられる単語の列全てを含むようなグラフ構造
struct LatticeGraph {
    graph: BTreeMap<i32, Vec<WordNode>>,
}
impl LatticeGraph {
    /// i文字目で終わるノードを探す
    fn node_list(&self, i: i32) -> Option<&Vec<WordNode>> {
        self.graph.get(&i)
    }

    // -1  0  1 2
    // BOS わ た し
    //     [  ][ ]
    //     [     ]
    fn get_prev_nodes(&self, node: &WordNode) -> Option<&Vec<WordNode>> {
        // ここの処理を簡単にするために BOS が入っている、のだとおもう。
        trace!("get_prev_nodes: {}", node.start_pos - 1);
        self.graph.get(&(node.start_pos))
    }

    fn get(&self, n: i32) -> Option<&Vec<WordNode>> {
        return self.graph.get(&n);
    }

    // for debugging purpose
    #[allow(unused)]
    fn dump(&self) {
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
    fn dump_dot(&self) -> String {
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
        buf += &*format!("}}\n");
        buf
    }

    fn get_node_cost(&self, node: &WordNode) -> f32 {
        // 簡単のために、一旦スコアを文字列長とする。
        // 経験上、長い文字列のほうがあたり、というルールでもそこそこ変換できる。
        // TODO あとでちゃんと unigram のコストを使うよに変える。

        return if node.kanji.len() < node.yomi.len() {
            // log10(1e-20)
            -20.0
        } else {
            // log10(1e-19)
            -19.0
        };

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

    fn get_edge_cost(&self, _prev: &WordNode, _node: &WordNode) -> f32 {
        // TODO: あとで実装する
        return 0.0;
    }
}

// 次に必要なのは、分割された文字列から、グラフを構築する仕組みである。
struct GraphResolver {}

impl GraphResolver {
    fn new() -> GraphResolver {
        GraphResolver {}
    }

    fn viterbi(&self, yomi: &String, lattice: LatticeGraph) -> String {
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
                let mut cost = f32::MAX;
                let mut shortest_prev = None;
                let prev_nodes = lattice.get_prev_nodes(&node).expect(
                    format!(
                        "Cannot get prev nodes for '{}' start={}",
                        node.kanji, node.start_pos
                    )
                    .as_str(),
                );
                for prev in prev_nodes.clone() {
                    let edge_cost = lattice.get_edge_cost(&prev, &node);
                    let prev_cost = costmap.get(prev).unwrap_or(&0_f32); // unwrap が必要なのは、 __BOS__ 用。
                    let tmp_cost = prev_cost + edge_cost + node_cost;
                    println!(
                        "Replace??? prev_cost={} tmp_cost={} < cost={}: {}",
                        prev_cost, tmp_cost, cost, prev
                    );
                    if tmp_cost < cost {
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
                .expect(format!("Cannot get previous node: {}", node.kanji).as_str());
        }
        result.reverse();
        return result.join("");
    }
}

#[cfg(test)]
mod tests {
    use crate::kana_kanji_dict::KanaKanjiDictBuilder;
    use crate::kana_trie::KanaTrieBuilder;
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    fn test() {
        let mut builder = KanaTrieBuilder::new();
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
        env_logger::builder().is_test(true).try_init().unwrap();

        let mut builder = KanaTrieBuilder::new();
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
        let dict_builder = KanaKanjiDictBuilder::new();
        let dict = dict_builder.build();
        let graph_builder = GraphBuilder::new(dict);
        let lattice = graph_builder.construct(&"abc".to_string(), graph);
        let resolver = GraphResolver::new();
        let result = resolver.viterbi(&"abc".to_string(), lattice);
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_kana_kanji() {
        env_logger::builder().is_test(true).try_init().unwrap();

        let mut builder = KanaTrieBuilder::new();
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

        let mut dict_builder = KanaKanjiDictBuilder::new();
        dict_builder.add("わたし", "私/渡し");

        let yomi = "わたし".to_string();

        let dict = dict_builder.build();
        let graph_builder = GraphBuilder::new(dict);
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
