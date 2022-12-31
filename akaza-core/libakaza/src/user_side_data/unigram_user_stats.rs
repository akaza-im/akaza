use std::collections::HashMap;

const ALPHA: f32 = 0.00001;

#[derive(Default)]
pub(crate) struct UniGramUserStats {
    /// ユニーク単語数
    unique_words: u32, // C
    /// 総単語出現数
    total_words: u32, // V
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
     * システム言語モデルと似ているがちょっと違う式を使ってる模様。
     */
    pub(crate) fn get_cost(&self, key: String) -> Option<f32> {
        let Some(count) = self.word_count.get(key.as_str()) else {
            return None;
        };

        Some(f32::log10(
            ((*count as f32) + ALPHA)
                / ((self.unique_words as f32) + ALPHA * (self.total_words as f32)),
        ))
    }

    pub(crate) fn record_entries(&mut self, kanji_kanas: &Vec<String>) {
        for kanji_kana in kanji_kanas {
            if let Some(i) = self.word_count.get(kanji_kana) {
                self.word_count.insert(kanji_kana.clone(), i + 1);
            } else {
                self.word_count.insert(kanji_kana.clone(), 1);
                self.unique_words += 1;
            }
            self.total_words += 1;
        }
    }
}
