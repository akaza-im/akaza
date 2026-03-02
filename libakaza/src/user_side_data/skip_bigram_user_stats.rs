use rustc_hash::FxHashMap;

use crate::cost::calc_cost;
use crate::graph::candidate::Candidate;
use crate::graph::word_node::BOS_TOKEN_KEY;
use crate::numeric_counter::normalize_counter_key_for_lm;

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
        if let Some(count) = self.word_count.get(key.as_str()) {
            return Some(calc_cost(*count, self.unique_words, self.total_words));
        }

        let norm1 = normalize_counter_key_for_lm(key1).unwrap_or_else(|| key1.to_string());
        let norm2 = normalize_counter_key_for_lm(key2).unwrap_or_else(|| key2.to_string());
        if norm1 == key1 && norm2 == key2 {
            return None;
        }

        let mut normalized = String::with_capacity(norm1.len() + 1 + norm2.len());
        normalized.push_str(&norm1);
        normalized.push('\t');
        normalized.push_str(&norm2);
        let count = self.word_count.get(normalized.as_str())?;
        Some(calc_cost(*count, self.unique_words, self.total_words))
    }

    fn record_pair(&mut self, key1: &str, key2: &str) {
        let key = format!("{}\t{}", key1, key2);
        if let Some(cnt) = self.word_count.get(&key) {
            self.word_count.insert(key, cnt + 1);
        } else {
            self.word_count.insert(key, 1);
            self.unique_words += 1;
        }
        self.total_words += 1;
    }

    /// candidates から skip-bigram ペア (i-2, i) を記録する。
    /// BOS を仮想的な位置 -1 として扱い、BOS→candidates[1] も記録する。
    pub(crate) fn record_entries(&mut self, candidates: &[Candidate]) {
        // BOS → candidates[1] (BOS が位置 -1、candidates[0] が位置 0、candidates[1] が位置 1)
        if candidates.len() >= 2 {
            let key2 =
                normalize_counter_key_for_lm(&candidates[1].key()).unwrap_or(candidates[1].key());
            self.record_pair(BOS_TOKEN_KEY, &key2);
        }

        // 通常の skip-bigram (i-2, i)
        for i in 2..candidates.len() {
            let key1 = normalize_counter_key_for_lm(&candidates[i - 2].key())
                .unwrap_or(candidates[i - 2].key());
            let key2 =
                normalize_counter_key_for_lm(&candidates[i].key()).unwrap_or(candidates[i].key());
            self.record_pair(&key1, &key2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bos_skip_bigram_with_two_candidates() {
        let mut stats = SkipBigramUserStats::default();
        // 2語: BOS→candidates[1] の skip-bigram が記録される
        stats.record_entries(&[
            Candidate::new("わたし", "私", 0.0),
            Candidate::new("は", "は", 0.0),
        ]);

        assert_eq!(stats.word_count.get("__BOS__/__BOS__\tは/は"), Some(&1));
        assert_eq!(stats.total_words, 1);
    }

    #[test]
    fn test_bos_skip_bigram_with_three_candidates() {
        let mut stats = SkipBigramUserStats::default();
        // 3語: BOS→candidates[1] と candidates[0]→candidates[2]
        stats.record_entries(&[
            Candidate::new("きょう", "今日", 0.0),
            Candidate::new("は", "は", 0.0),
            Candidate::new("いい", "良い", 0.0),
        ]);

        // BOS → は (skip-bigram)
        assert_eq!(stats.word_count.get("__BOS__/__BOS__\tは/は"), Some(&1));
        // 今日 → 良い (通常 skip-bigram)
        assert_eq!(stats.word_count.get("今日/きょう\t良い/いい"), Some(&1));
        assert_eq!(stats.total_words, 2);
    }

    #[test]
    fn test_single_candidate_no_skip_bigram() {
        let mut stats = SkipBigramUserStats::default();
        // 1語では skip-bigram は記録されない
        stats.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);
        assert_eq!(stats.total_words, 0);
    }

    #[test]
    fn test_empty_candidates() {
        let mut stats = SkipBigramUserStats::default();
        stats.record_entries(&[]);
        assert_eq!(stats.total_words, 0);
    }
}
