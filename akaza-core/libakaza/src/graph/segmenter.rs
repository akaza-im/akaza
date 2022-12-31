use std::collections::btree_map::Iter;
use std::collections::{BTreeMap, HashSet};

use log::trace;

use crate::kana_trie::KanaTrie;

#[derive(PartialEq, Debug)]
pub struct SegmentationResult {
    base: BTreeMap<usize, Vec<String>>,
}

impl SegmentationResult {
    pub(crate) fn new(base: BTreeMap<usize, Vec<String>>) -> SegmentationResult {
        SegmentationResult { base }
    }

    pub(crate) fn iter(&self) -> Iter<'_, usize, Vec<String>> {
        self.base.iter()
    }

    pub fn dump_dot(&self) -> String {
        let mut buf = String::new();
        buf += "digraph Lattice {\n";
        // start 及び end は、byte 数単位
        for (end_pos, yomis) in self.base.iter() {
            for yomi in yomis {
                buf += &*format!(r#"    {} -> "{}"{}"#, end_pos - yomi.len(), yomi, "\n");
                buf += &*format!(r#"    {} -> "{}"{}"#, yomi, end_pos, "\n");
            }
        }
        buf += &*"}\n".to_string();
        buf
    }
}

pub struct Segmenter {
    tries: Vec<KanaTrie>,
}

impl Segmenter {
    pub fn new(tries: Vec<KanaTrie>) -> Segmenter {
        Segmenter { tries }
    }

    /**
     * 読みをうけとって、グラフを構築する。
     */
    // シフトを押して → を押したときのような処理の場合、
    // このメソッドに入ってくる前に別に処理する前提。
    pub fn build(&self, yomi: &str) -> SegmentationResult {
        let mut queue: Vec<usize> = Vec::new(); // 検索対象となる開始位置
        queue.push(0);
        let mut seen: HashSet<usize> = HashSet::new();

        // 終了位置ごとの候補単語リスト
        let mut words_ends_at: BTreeMap<usize, Vec<String>> = BTreeMap::new();

        while !queue.is_empty() {
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
            if !candidates.is_empty() {
                for candidate in &candidates {
                    let ends_at = start_pos + candidate.len();

                    let vec = words_ends_at.entry(ends_at).or_default();
                    vec.push(candidate.clone());

                    queue.push(start_pos + candidate.len());
                }
            } else {
                // 辞書に1文字も候補がない場合は先頭文字を取り出してグラフに入れる
                // ここは改善の余地がありそう。

                trace!("There's no candidates. '{}'", yomi);

                let (_, c) = yomi.char_indices().next().unwrap();
                let first = &yomi[0..c.len_utf8()];
                let ends_at = start_pos + first.len();

                let vec = words_ends_at.entry(ends_at).or_default();
                vec.push(first.to_string());

                queue.push(start_pos + first.len())
            }
        }

        SegmentationResult {
            base: words_ends_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kana_trie::KanaTrieBuilder;

    use super::*;

    #[test]
    fn test_simple() {
        let mut builder = KanaTrieBuilder::default();
        builder.add(&"わたし".to_string());
        builder.add(&"わた".to_string());
        builder.add(&"し".to_string());
        let kana_trie = builder.build();

        let segmenter = Segmenter::new(vec![kana_trie]);
        let graph = segmenter.build("わたし");
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ]))
        )
    }

    #[test]
    fn test_without_kanatrie() {
        let builder = KanaTrieBuilder::default();
        let kana_trie = builder.build();

        let segmenter = Segmenter::new(vec![kana_trie]);
        let graph = segmenter.build("わたし");
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (3, vec!["わ".to_string()]),
                (6, vec!["た".to_string()]),
                (9, vec!["し".to_string()]),
            ]))
        )
    }
}
