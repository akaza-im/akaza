use std::collections::HashMap;

const ALPHA: f32 = 0.00001;

pub(crate) struct UniGramUserStats {
    /// ユニーク単語数
    unique_words: u32, // C
    /// 総単語出現数
    total_words: u32, // V
    /// その単語の出現頻度。「漢字」がキー。
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
     * システム言語モデルのコストよりも安くなるように調整してある。
     */
    pub(crate) fn get_cost(&self, key: &String) -> Option<f32> {
        let Some(count) = self.word_count.get(key) else {
            return None;
        };
        return Some(f32::log10(
            ((*count as f32) + ALPHA)
                / ((self.unique_words as f32) + ALPHA + (self.total_words as f32)),
        ));
    }

    pub(crate) fn record_entries(&mut self, kanjis: &Vec<String>) {
        for kanji in kanjis {
            if let Some(i) = self.word_count.get(kanji) {
                self.word_count.insert(kanji.clone(), i + 1);
            } else {
                self.word_count.insert(kanji.clone(), 1);
                self.unique_words += 1;
            }
            self.total_words += 1;
        }
    }
}
