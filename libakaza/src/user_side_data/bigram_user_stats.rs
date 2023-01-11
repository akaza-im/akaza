use std::collections::HashMap;

const ALPHA: f32 = 0.00001;

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
        Some(-f32::log10(
            ((*count as f32) + ALPHA)
                / ((self.unique_words as f32) + ALPHA + (self.total_words as f32)),
        ))
    }

    pub(crate) fn record_entries(&mut self, kanjis: &Vec<String>) {
        if kanjis.len() < 2 {
            return;
        }

        // bigram
        for i in 1..kanjis.len() {
            let Some(kanji1) = kanjis.get(i - 1) else {
                continue;
            };
            let Some(kanji2) = kanjis.get(i) else {
                continue;
            };

            let key = kanji1.clone() + "\t" + kanji2;
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
