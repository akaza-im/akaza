use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use marisa_sys::{Keyset, Marisa};

// const BOS: WordNode = WordNode::create_bos();
// const EOS: WordNode = WordNode::create_eos();

#[derive(PartialEq)]
struct WordNode {
    start_pos: usize,
    kanji: String,
    cost: f32,
    prev: Option<Rc<RefCell<WordNode>>,
}

impl WordNode {
    fn create_bos() -> WordNode {
        WordNode {
            start_pos: 0,
            kanji: "__BOS__".to_string(),
            cost: 0_f32,
            prev: None,
        }
    }
    fn create_eos() -> WordNode {
        WordNode {
            start_pos: 0,
            kanji: "__EOS__".to_string(),
            cost: 0_f32,
            prev: None,
        }
    }
    fn new(start_pos: usize, kanji: &String) -> WordNode {
        WordNode {
            start_pos,
            kanji: kanji.clone(),
            cost: 0_f32,
            prev: None,
        }
    }

    pub fn set_prev(&mut self, prev: Option<Rc<RefCell<WordNode>>>) {
        match prev {
            Some(thing) => self.prev = Some(Rc::clone(&thing)),
            None => self.prev = None,
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
    graph: HashMap<usize, Vec<Rc<RefCell<WordNode>>>>,
}

impl GraphResolver {
    fn new(yomi: &String, words_ends_at: HashMap<usize, Vec<String>>) -> GraphResolver {
        let graph = Self::construct_graph(yomi, words_ends_at);
        GraphResolver { graph }
    }

    fn construct_graph(
        yomi: &String,
        words_ends_at: HashMap<usize, Vec<String>>,
    ) -> HashMap<usize, Vec<Rc<RefCell<WordNode>>>> {
        let mut graph: HashMap<usize, Vec<Rc<RefCell<WordNode>>>> = HashMap::new();
        graph.insert(0, vec![Rc::new(RefCell::new(WordNode::create_bos()))]);
        graph.insert(
            yomi.len() + 1,
            vec![Rc::new(RefCell::new(WordNode::create_eos()))],
        );

        for (end_pos, yomis) in words_ends_at {
            for yomi in yomis {
                // ↓ここで start_pos 渡す意味ある?
                let node = WordNode::new(end_pos - yomi.len(), &yomi);
                // ほんとうは、そもそもここで、漢字に変換した結果を詰め込まないといけない。

                let vec = graph.entry(end_pos).or_insert(Vec::new());
                vec.push(Rc::new(RefCell::new(node)));
            }
        }
        return graph;
    }

    /// i文字目で終わるノードを探す
    fn node_list(&self, i: usize) -> Option<Vec<Rc<RefCell<WordNode>>>> {
        match self.graph.get(&i) {
            Some(p) => Some(p.clone()),
            None => None,
        }
    }

    fn get_node_cost(&self, node: &Rc<RefCell<WordNode>>) -> f32 {
        // 簡単のために、一旦スコアを文字列長とする。
        // 経験上、長い文字列のほうがあたり、というルールでもそこそこ変換できる。
        // TODO あとでちゃんと unigram のコストを使うよに変える。
        // ここ明らかに書き方おかしいと思う。
        unsafe {
            return (*(node.as_ptr())).kanji.len() as f32;
        }
    }

    fn get_prev_nodes(&self, node: &Rc<RefCell<WordNode>>) -> Option<&Vec<Rc<RefCell<WordNode>>>> {
        // ここの処理を簡単にするために BOS が入っている、のだとおもう。
        self.graph.get(&(node.as_ref().borrow().start_pos - 1))
    }

    fn get_edge_cost(&self, prev: &Rc<RefCell<WordNode>>, node: &Rc<RefCell<WordNode>>) -> f32 {
        // TODO: あとで実装する
        return 0.0;
    }

    fn viterbi(&self, yomi: &String) {
        for i in 0..yomi.len() {
            let Some(nodes) = self.node_list(i) else {
                continue;
            };
            for mut node in nodes {
                let node_cost = self.get_node_cost(&node);
                let mut cost = f32::MAX;
                let mut shortest_prev = None;
                let prev_nodes = self.get_prev_nodes(&node).unwrap();
                unsafe {
                    for prev in prev_nodes.clone() {
                        let edge_cost = self.get_edge_cost(&prev, &node);
                        let tmp_cost = (*prev.as_ptr()).cost + edge_cost + node_cost;
                        if tmp_cost < cost {
                            cost = tmp_cost;
                            shortest_prev = Some(prev);
                        }
                    }
                }
                node.get_mut().set_prev(shortest_prev);

                let b = node.borrow_mut();
                // let mut p = node.borrow_mut();
                // p.get_mut().set_prev(shortest_prev);
                // p.get_mut().cost = cost;
            }
        }
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
}
