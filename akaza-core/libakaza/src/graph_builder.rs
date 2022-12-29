use marisa_sys::{Keyset, Marisa};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/**
 * 有向グラフ。文のうしろから前に向かってリンクされる。
 */
#[derive(PartialEq, Debug)]
struct GraphNode {
    yomi: String,
    // 一個前のノード
    prev: Option<Rc<RefCell<GraphNode>>>,
}

impl GraphNode {
    fn new(yomi: &String, prev: Option<Rc<RefCell<GraphNode>>>) -> GraphNode {
        GraphNode {
            yomi: yomi.clone(),
            prev,
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

struct GraphBuilder {
    kana_trie: KanaTrie,
}

impl GraphBuilder {
    pub(crate) fn new(kana_trie: KanaTrie) -> GraphBuilder {
        GraphBuilder { kana_trie }
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

        let mut words_start_at: HashMap<usize, Vec<String>> = HashMap::new();

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
                let mut results: Vec<String> = Vec::new();

                for candidate in &candidates {
                    results.push(candidate.clone());

                    queue.push(start_pos + candidate.len());
                }
                words_start_at.insert(start_pos, results);
            } else {
                // 辞書に1文字も候補がない場合は先頭文字を取り出してグラフに入れる
                // ここは改善の余地がありそう。

                let (i, _) = yomi.char_indices().nth(1).unwrap();
                let first = &yomi[0..i];

                words_start_at.insert(start_pos, vec![first.to_string()]);
                queue.push(start_pos + first.len())
            }
        }

        return words_start_at;
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

        let graph_builder = GraphBuilder::new(kana_trie);
        let graph = graph_builder.build(&"わたし".to_string());
        assert_eq!(
            graph,
            HashMap::from([
                (0, vec!["わた".to_string(), "わたし".to_string()]),
                (6, vec!["し".to_string()])
            ])
        )
    }
}
