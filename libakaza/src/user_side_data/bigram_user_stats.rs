use rustc_hash::FxHashMap;

use crate::cost::calc_cost;
use crate::graph::candidate::Candidate;
use crate::graph::word_node::{BOS_TOKEN_KEY, EOS_TOKEN_KEY};
use crate::numeric_counter::normalize_counter_key_for_lm;

#[derive(Default)]
pub(crate) struct BiGramUserStats {
    /// ユニーク単語数
    unique_words: u32,
    // C
    /// 総単語出現数
    total_words: u32,
    // V
    /// その単語の出現頻度。「漢字/漢字」がキー。
    pub(crate) word_count: FxHashMap<String, u32>,
}

impl BiGramUserStats {
    pub(crate) fn new(
        unique_words: u32,
        total_words: u32,
        word_count: FxHashMap<String, u32>,
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
        // ユーザー統計が空なら、キー構築・数字正規化を行わず即座に None を返す。
        // Viterbi DP のエッジ評価から呼ばれるため、空マップ時の無駄な alloc/パースを避ける。
        if self.word_count.is_empty() {
            return None;
        }

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

    pub(crate) fn record_entries(&mut self, candidates: &[Candidate]) {
        if candidates.is_empty() {
            return;
        }

        // BOS → 先頭候補
        let first_key =
            normalize_counter_key_for_lm(&candidates[0].key()).unwrap_or(candidates[0].key());
        self.record_pair(BOS_TOKEN_KEY, &first_key);

        // 隣接 bigram
        for i in 1..candidates.len() {
            let key1 = normalize_counter_key_for_lm(&candidates[i - 1].key())
                .unwrap_or(candidates[i - 1].key());
            let key2 =
                normalize_counter_key_for_lm(&candidates[i].key()).unwrap_or(candidates[i].key());
            self.record_pair(&key1, &key2);
        }

        // 末尾候補 → EOS
        let last_key = normalize_counter_key_for_lm(&candidates.last().unwrap().key())
            .unwrap_or(candidates.last().unwrap().key());
        self.record_pair(&last_key, EOS_TOKEN_KEY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_bos_eos_single_candidate() {
        let mut stats = BiGramUserStats::default();
        // 1語の確定: BOS→互換→EOS
        stats.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);

        assert_eq!(
            stats.word_count.get("__BOS__/__BOS__\t互換/ごかん"),
            Some(&1)
        );
        assert_eq!(
            stats.word_count.get("互換/ごかん\t__EOS__/__EOS__"),
            Some(&1)
        );
        assert_eq!(stats.total_words, 2); // BOS→互換, 互換→EOS
        assert_eq!(stats.unique_words, 2);
    }

    #[test]
    fn test_record_bos_eos_multi_candidates() {
        let mut stats = BiGramUserStats::default();
        // 3語の確定: BOS→今日→は→良い→EOS
        stats.record_entries(&[
            Candidate::new("きょう", "今日", 0.0),
            Candidate::new("は", "は", 0.0),
            Candidate::new("いい", "良い", 0.0),
        ]);

        // BOS→今日
        assert_eq!(
            stats.word_count.get("__BOS__/__BOS__\t今日/きょう"),
            Some(&1)
        );
        // 今日→は
        assert_eq!(stats.word_count.get("今日/きょう\tは/は"), Some(&1));
        // は→良い
        assert_eq!(stats.word_count.get("は/は\t良い/いい"), Some(&1));
        // 良い→EOS
        assert_eq!(stats.word_count.get("良い/いい\t__EOS__/__EOS__"), Some(&1));
        assert_eq!(stats.total_words, 4); // BOS→今日, 今日→は, は→良い, 良い→EOS
    }

    #[test]
    fn test_bos_bigram_accumulates() {
        let mut stats = BiGramUserStats::default();
        // 「互換」を3回確定
        for _ in 0..3 {
            stats.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);
        }

        assert_eq!(
            stats.word_count.get("__BOS__/__BOS__\t互換/ごかん"),
            Some(&3)
        );
        // コスト計算: 3回選択のコスト < 1回選択のコスト
        let cost_3 = stats.get_cost("__BOS__/__BOS__", "互換/ごかん").unwrap();
        let mut stats2 = BiGramUserStats::default();
        stats2.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);
        let cost_1 = stats2.get_cost("__BOS__/__BOS__", "互換/ごかん").unwrap();
        assert!(cost_3 < cost_1);
    }

    #[test]
    fn test_bos_bigram_differentiates_candidates() {
        let mut stats = BiGramUserStats::default();
        // 「互換」を5回、「五感」を2回確定
        for _ in 0..5 {
            stats.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);
        }
        for _ in 0..2 {
            stats.record_entries(&[Candidate::new("ごかん", "五感", 0.0)]);
        }

        let cost_gokan = stats.get_cost("__BOS__/__BOS__", "互換/ごかん").unwrap();
        let cost_gokan2 = stats.get_cost("__BOS__/__BOS__", "五感/ごかん").unwrap();
        // 互換(5回) のほうが五感(2回) よりコストが低い
        assert!(
            cost_gokan < cost_gokan2,
            "互換({}) should be cheaper than 五感({})",
            cost_gokan,
            cost_gokan2
        );
    }

    #[test]
    fn test_empty_candidates() {
        let mut stats = BiGramUserStats::default();
        stats.record_entries(&[]);
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.unique_words, 0);
    }

    #[test]
    fn test_adjacent_bigram_still_works() {
        let mut stats = BiGramUserStats::default();
        stats.record_entries(&[
            Candidate::new("わたし", "私", 0.0),
            Candidate::new("は", "は", 0.0),
        ]);

        // 従来の隣接 bigram も記録されている
        assert_eq!(stats.word_count.get("私/わたし\tは/は"), Some(&1));
        // BOS/EOS も記録されている
        assert_eq!(stats.word_count.get("__BOS__/__BOS__\t私/わたし"), Some(&1));
        assert_eq!(stats.word_count.get("は/は\t__EOS__/__EOS__"), Some(&1));
    }
}
