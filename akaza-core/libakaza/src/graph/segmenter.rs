use log::trace;

use std::collections::{HashMap, HashSet};




use crate::kana_trie::KanaTrie;



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
    pub fn build(&self, yomi: &str) -> HashMap<usize, Vec<String>> {
        let mut queue: Vec<usize> = Vec::new(); // 検索対象となる開始位置
        queue.push(0);
        let mut seen: HashSet<usize> = HashSet::new();

        // 終了位置ごとの候補単語リスト
        let mut words_ends_at: HashMap<usize, Vec<String>> = HashMap::new();

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

                let (i, _) = yomi.char_indices().nth(1).unwrap();
                let first = &yomi[0..i];
                let ends_at = start_pos + first.len();

                let vec = words_ends_at.entry(ends_at).or_default();
                vec.push(first.to_string());

                queue.push(start_pos + first.len())
            }
        }

        words_ends_at
    }
}