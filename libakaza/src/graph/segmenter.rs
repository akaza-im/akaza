use std::collections::btree_map::{BTreeMap, Iter};
use std::collections::HashSet;
use std::ops::Range;

use log::trace;

use crate::kana_trie::base::KanaTrie;

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
    tries: Vec<Box<dyn KanaTrie>>,
}

impl Segmenter {
    pub fn new(tries: Vec<Box<dyn KanaTrie>>) -> Segmenter {
        Segmenter { tries }
    }

    /**
     * 「読み」を受け取って Lattice を構築する。
     *
     * force_ranges: 一般的な IME でシフトおしてから→をおして、ユーザーが明示的に範囲選択した場合
     *               の選択範囲。
     */
    // シフトを押して → を押したときのような処理の場合、
    // このメソッドに入ってくる前に別に処理する前提。
    pub fn build(&self, yomi: &str, force_ranges: Option<&[Range<usize>]>) -> SegmentationResult {
        if let Some(force_ranges) = force_ranges {
            if !force_ranges.is_empty() {
                for force_range in force_ranges {
                    trace!(
                        "force_range detected: {}",
                        yomi[force_range.start..force_range.end].to_string()
                    );
                }
            }
        }

        let mut queue: Vec<usize> = Vec::new(); // 検索対象となる開始位置
        queue.push(0);
        let mut seen: HashSet<usize> = HashSet::new();

        // 終了位置ごとの候補単語リスト
        let mut words_ends_at: BTreeMap<usize, Vec<String>> = BTreeMap::new();

        'queue_processing: while !queue.is_empty() {
            let start_pos = queue.pop().unwrap();
            if seen.contains(&start_pos) {
                continue;
            } else {
                seen.insert(start_pos);
            }

            // start_pos が force の範囲に入っていたら処理しない。
            if let Some(force_ranges) = force_ranges {
                for force_range in force_ranges {
                    if force_range.start == start_pos {
                        trace!("force_range detected.");
                        let vec = words_ends_at.entry(force_range.end).or_default();
                        vec.push(yomi[force_range.start..force_range.end].to_string());
                        queue.push(start_pos + force_range.len());
                        continue 'queue_processing;
                    }
                    if force_range.contains(&start_pos) {
                        continue 'queue_processing;
                    }
                }
            }

            let yomi = &yomi[start_pos..];
            if yomi.is_empty() {
                continue;
            }

            let mut candidates: HashSet<String> = HashSet::new();
            for trie in &self.tries {
                let got = trie.common_prefix_search(yomi);
                'insert: for word in got {
                    let ends_at = start_pos + word.len();

                    // end_pos が force の範囲に入っていたら処理しない。
                    if let Some(force_ranges) = force_ranges {
                        for force_range in force_ranges {
                            // force_range は exclusive で、厳しい。
                            if force_range.contains(&ends_at) || force_range.end == ends_at {
                                trace!("Blocked candidate range: {}, {:?}", word, force_range);
                                continue 'insert;
                            } else {
                                trace!("Accepted candidate range: {}, {:?}", word, force_range);
                            }
                        }
                    }

                    candidates.insert(word);
                }
            }
            if !candidates.is_empty() {
                for candidate in &candidates {
                    let ends_at = start_pos + candidate.len();

                    let vec = words_ends_at.entry(ends_at).or_default();
                    trace!("Add candidate: {}", candidate);
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
    use crate::kana_trie::marisa_kana_trie::MarisaKanaTrie;

    use super::*;

    #[test]
    fn test_simple() {
        let kana_trie = MarisaKanaTrie::build(vec![
            "わたし".to_string(),
            "わた".to_string(),
            "し".to_string(),
        ]);

        let segmenter = Segmenter::new(vec![Box::new(kana_trie)]);
        let graph = segmenter.build("わたし", None);
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
        let kana_trie = MarisaKanaTrie::build(vec![]);

        let segmenter = Segmenter::new(vec![Box::new(kana_trie)]);
        let graph = segmenter.build("わたし", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (3, vec!["わ".to_string()]),
                (6, vec!["た".to_string()]),
                (9, vec!["し".to_string()]),
            ]))
        )
    }

    #[test]
    fn test_force() -> anyhow::Result<()> {
        // env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
        // env_logger::builder().is_test(true).try_init()?;

        let kana_trie = MarisaKanaTrie::build(Vec::from([
            "わたし".to_string(),
            "わた".to_string(),
            "わ".to_string(),
            "し".to_string(),
        ]));

        let segmenter = Segmenter::new(vec![Box::new(kana_trie)]);
        let yomi = "わたし";
        // force_range に "たし" を指定する。
        let (i2, _) = yomi.char_indices().nth(1).unwrap();
        let (i3, c3) = yomi.char_indices().nth(2).unwrap();
        let graph = segmenter.build(yomi, Some(&[i2..(i3 + c3.len_utf8())]));
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (3, vec!["わ".to_string()]),
                (9, vec!["たし".to_string()]),
            ]))
        );
        Ok(())
    }
}
