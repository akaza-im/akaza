use log::trace;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use marisa_sys::{Keyset, Marisa};

#[derive(PartialEq)]
struct WordNode {
    start_pos: i32,
    kanji: String,
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
            cost: 0_f32,
        }
    }
    fn create_eos(yomi: &String) -> WordNode {
        WordNode {
            start_pos: yomi.len() as i32,
            kanji: "__EOS__".to_string(),
            cost: 0_f32,
        }
    }
    fn new(start_pos: i32, kanji: &String) -> WordNode {
        WordNode {
            start_pos,
            kanji: kanji.clone(),
            cost: 0_f32,
        }
    }
}

struct KanaTrieBuilder {
    keyset: Keyset,
}

impl KanaTrieBuilder {
    fn new() -> KanaTrieBuilder {
        KanaTrieBuilder {
            keyset: Keyset::new(),
        }
    }

    fn add(&mut self, yomi: &String) {
        self.keyset.push_back(yomi.as_bytes());
    }

    fn build(&self) -> KanaTrie {
        let marisa = Marisa::new();
        marisa.build(&self.keyset);
        KanaTrie { marisa }
    }
}

struct KanaTrie {
    marisa: Marisa,
}

impl KanaTrie {
    pub(crate) fn common_prefix_search(&self, query: &String) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        self.marisa.common_prefix_search(query, |word, _| {
            result.push(String::from_utf8(word.to_vec()).unwrap());
            true
        });
        return result;
    }

    fn save(&self, file_name: &String) -> Result<(), String> {
        self.marisa.save(file_name)
    }

    fn load(file_name: &String) -> KanaTrie {
        let marisa = Marisa::new();
        marisa.load(file_name).unwrap();
        KanaTrie { marisa }
    }
}

struct Segmenter {
    kana_trie: KanaTrie,
}

impl Segmenter {
    pub(crate) fn new(kana_trie: KanaTrie) -> Segmenter {
        Segmenter { kana_trie }
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

            let candidates = self.kana_trie.common_prefix_search(&yomi.to_string());
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

// 次に必要なのは、分割された文字列から、グラフを構築する仕組みである。
struct GraphResolver {
    graph: HashMap<i32, Vec<WordNode>>,
}

impl GraphResolver {
    fn new(yomi: &String, words_ends_at: HashMap<usize, Vec<String>>) -> GraphResolver {
        let graph = Self::construct_graph(yomi, words_ends_at);
        GraphResolver { graph }
    }

    fn construct_graph(
        yomi: &String,
        words_ends_at: HashMap<usize, Vec<String>>,
    ) -> HashMap<i32, Vec<WordNode>> {
        // このグラフのインデクスは単語の終了位置。
        let mut graph: HashMap<i32, Vec<WordNode>> = HashMap::new();
        graph.insert(0, vec![WordNode::create_bos()]);
        graph.insert((yomi.len() + 1) as i32, vec![WordNode::create_eos(yomi)]);

        for (end_pos, yomis) in words_ends_at {
            for yomi in yomis {
                // ↓ここで start_pos 渡す意味ある?
                let node = WordNode::new((end_pos - yomi.len()) as i32, &yomi);
                // ほんとうは、そもそもここで、漢字に変換した結果を詰め込まないといけない。

                let vec = graph.entry(end_pos as i32).or_insert(Vec::new());
                vec.push(node);
            }
        }
        return graph;
    }

    /// i文字目で終わるノードを探す
    fn node_list(&self, i: i32) -> Option<&Vec<WordNode>> {
        self.graph.get(&i)
    }

    fn get_node_cost(&self, node: &WordNode) -> f32 {
        // 簡単のために、一旦スコアを文字列長とする。
        // 経験上、長い文字列のほうがあたり、というルールでもそこそこ変換できる。
        // TODO あとでちゃんと unigram のコストを使うよに変える。
        return node.kanji.len() as f32;
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

    fn get_edge_cost(&self, prev: &WordNode, node: &WordNode) -> f32 {
        // TODO: あとで実装する
        return 0.0;
    }

    fn viterbi(&self, yomi: &String) -> String {
        let mut prevmap: HashMap<&WordNode, &WordNode> = HashMap::new();

        for i in 1..yomi.len() + 2 {
            let Some(nodes) = self.node_list(i as i32) else {
                continue;
            };
            for node in nodes {
                let node_cost = self.get_node_cost(node);
                let mut cost = f32::MAX;
                let mut shortest_prev = None;
                let prev_nodes = self.get_prev_nodes(&node).expect(
                    format!(
                        "Cannot get prev nodes for '{}' start={}",
                        node.kanji, node.start_pos
                    )
                    .as_str(),
                );
                for prev in prev_nodes.clone() {
                    let edge_cost = self.get_edge_cost(&prev, &node);
                    let tmp_cost = prev.cost + edge_cost + node_cost;
                    if tmp_cost < cost {
                        cost = tmp_cost;
                        shortest_prev = Some(prev);
                    }
                }
                prevmap.insert(node, shortest_prev.unwrap());
            }
        }

        let eos = self
            .graph
            .get(&((yomi.len() + 1) as i32))
            .unwrap()
            .get(0)
            .unwrap();
        let bos = self.graph.get(&0).unwrap().get(0).unwrap();
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
    use super::*;

    #[test]
    fn test() {
        let mut builder = KanaTrieBuilder::new();
        builder.add(&"わたし".to_string());
        builder.add(&"わた".to_string());
        builder.add(&"し".to_string());
        let kana_trie = builder.build();

        let graph_builder = Segmenter::new(kana_trie);
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

        let graph_builder = Segmenter::new(kana_trie);
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
        let resolver = GraphResolver::new(&"abc".to_string(), graph);
        let result = resolver.viterbi(&"abc".to_string());
        assert_eq!(result, "abc");
    }
}
