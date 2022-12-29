use marisa_sys::{Keyset, Marisa};
use std::cell::RefCell;
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
}

impl KanaTrie {
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
    fn build(&self, yomi: &String) -> Vec<GraphNode> {
        assert_ne!(yomi.len(), 0);

        // TODO BOS ノード使う?
        self._build(yomi, None)
    }

    fn _build(&self, yomi: &String, prev: Option<Rc<RefCell<GraphNode>>>) -> Vec<GraphNode> {
        if yomi.len() == 0 {
            return vec![];
        }

        let candidates = self.kana_trie.common_prefix_search(yomi);
        return if candidates.len() == 0 {
            // 辞書に1文字も候補がない場合は先頭文字を取り出してグラフに入れる
            let (i, _) = yomi.char_indices().nth(1).unwrap();
            let first = &yomi[0..i];

            let current = GraphNode::new(&first.to_string(), prev);

            if yomi.len() == 1 {
                // 継続文字はない。current が終端ノード。
                vec![current]
            } else {
                // 続きの文字があるのでそれを処理してもらう
                let (i, _) = yomi.char_indices().nth(1).unwrap();
                let remains = &yomi[..i];

                self._build(&remains.to_string(), Some(Rc::new(RefCell::new(current))))
            }
        } else {
            // 辞書の候補をそれぞれケアする
            let mut result: Vec<GraphNode> = Vec::new();

            for candidate in &candidates {
                let current = GraphNode::new(&candidate.to_string(), prev.clone());

                if yomi.len() == candidate.len() {
                    // 継続文字はない。current が終端ノード。
                    result.push(current);
                } else {
                    // 続きの文字があるのでそれを処理してもらう
                    let (i, _) = yomi.char_indices().nth(candidate.chars().count()).unwrap();
                    let remains = &yomi[i..];
                    let got =
                        self._build(&remains.to_string(), Some(Rc::new(RefCell::new(current))));
                    for x in got {
                        result.push(x);
                    }
                }
            }
            result
        };
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
            vec![
                GraphNode::new(
                    &"し".to_string(),
                    Some(Rc::new(RefCell::new(GraphNode::new(
                        &"わた".to_string(),
                        None
                    ))))
                ),
                GraphNode::new(&"わたし".to_string(), None)
            ]
        );
    }
}
