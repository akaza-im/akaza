use std::collections::HashMap;

use crate::cost::calc_cost;
use crate::graph::candidate::Candidate;

#[derive(Default)]
pub(crate) struct BiGramUserStats {
    /// ユニーク単語数
    unique_words: u32,
    // C
    /// 総単語出現数
    total_words: u32,
    // V
    /// その単語の出現頻度。「漢字/漢字」がキー。
    pub(crate) word_count: HashMap<String, u32>,
}

impl BiGramUserStats {
    pub(crate) fn new(
        unique_words: u32,
        total_words: u32,
        word_count: HashMap<String, u32>,
    ) -> BiGramUserStats {
        BiGramUserStats {
            unique_words,
            total_words,
            word_count,
        }
    }

    /**
     * エッジコストを計算する。
     * システム言語モデルのコストよりも安くなるように調整してある。
     */
    pub(crate) fn get_cost(&self, key1: &str, key2: &str) -> Option<f32> {
        let key = key1.to_owned() + "\t" + key2;
        let Some(count) = self.word_count.get(key.as_str()) else {
            return None;
        };
        Some(calc_cost(*count, self.unique_words, self.total_words))
    }

    pub(crate) fn record_entries(&mut self, candidates: &[Candidate]) {
        if candidates.len() < 2 {
            return;
        }

        // bigram
        for i in 1..candidates.len() {
            let Some(candidate1) = candidates.get(i - 1) else {
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
