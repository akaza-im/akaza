use std::collections::HashMap;

use crate::cost::calc_cost;
use crate::graph::candidate::Candidate;

#[derive(Default)]
pub(crate) struct UniGramUserStats {
    /// ユニーク単語数
    unique_words: u32,
    // C
    /// 総単語出現数
    total_words: u32,
    // V
    /// その単語の出現頻度。「漢字/かな」がキー。
    pub(crate) word_count: HashMap<String, u32>,
}

impl UniGramUserStats {
    pub(crate) fn new(
        unique_words: u32,
        total_words: u32,
        word_count: HashMap<String, u32>,
    ) -> UniGramUserStats {
        UniGramUserStats {
            unique_words,
            total_words,
            word_count,
        }
    }

    /**
     * ノードコストを計算する。
     */
    pub(crate) fn get_cost(&self, key: String) -> Option<f32> {
        let count = self.word_count.get(key.as_str())?;

        Some(calc_cost(*count, self.unique_words, self.total_words))
    }

    pub(crate) fn record_entries(&mut self, candidates: &[Candidate]) {
        for candidate in candidates {
            let key = candidate.key();
            if let Some(i) = self.word_count.get(&key) {
                self.word_count.insert(key, i + 1);
            } else {
                self.word_count.insert(key, 1);
                self.unique_words += 1;
            }
            self.total_words += 1;
        }
    }
}
