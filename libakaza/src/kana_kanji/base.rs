pub trait KanaKanjiDict {
    fn get(&self, kana: &str) -> Option<Vec<String>>;
}
