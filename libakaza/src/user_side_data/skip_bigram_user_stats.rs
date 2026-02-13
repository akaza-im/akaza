use rustc_hash::FxHashMap;

use crate::cost::calc_cost;
use crate::graph::candidate::Candidate;

#[derive(Default)]
pub(crate) struct SkipBigramUserStats {
    /// ユニーク単語数
    unique_words: u32,
    /// 総単語出現数
    total_words: u32,
    /// skip-bigram の出現頻度。"surface1/kana1\tsurface2/kana2" がキー。
    pub(crate) word_count: FxHashMap<String, u32>,
}

impl SkipBigramUserStats {
    pub(crate) fn new(
        unique_words: u32,
        total_words: u32,
        word_count: FxHashMap<String, u32>,
    ) -> SkipBigramUserStats {
        SkipBigramUserStats {
            unique_words,
            total_words,
            word_count,
        }
    }

    /// skip-bigram のエッジコストを計算する。
    pub(crate) fn get_cost(&self, key1: &str, key2: &str) -> Option<f32> {
        let mut key = String::with_capacity(key1.len() + 1 + key2.len());
        key.push_str(key1);
        key.push('\t');
        key.push_str(key2);
        let count = self.word_count.get(key.as_str())?;
        Some(calc_cost(*count, self.unique_words, self.total_words))
    }

    /// candidates から skip-bigram ペア (i-2, i) を記録する。
    pub(crate) fn record_entries(&mut self, candidates: &[Candidate]) {
        if candidates.len() < 3 {
            return;
        }

        for i in 2..candidates.len() {
            let Some(candidate1) = candidates.get(i - 2) else {
                continue;
            };
            let Some(candidate2) = candidates.get(i) else {
                continue;
            };

            let key = candidate1.key() + "\t" + candidate2.key().as_str();
            if let Some(cnt) = self.word_count.get(&key) {
                self.word_count.insert(key, cnt + 1);
            } else {
                self.word_count.insert(key, 1);
                self.unique_words += 1;
            }
            self.total_words += 1;
        }
    }
}
