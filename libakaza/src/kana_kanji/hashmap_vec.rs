use std::collections::HashMap;

use crate::kana_kanji::base::KanaKanjiDict;

#[derive(Default)]
pub struct HashmapVecKanaKanjiDict {
    map: HashMap<String, Vec<String>>,
}

impl HashmapVecKanaKanjiDict {
    pub fn new(map: HashMap<String, Vec<String>>) -> Self {
        HashmapVecKanaKanjiDict { map }
    }
}

impl KanaKanjiDict for HashmapVecKanaKanjiDict {
    fn get(&self, kana: &str) -> Option<Vec<String>> {
        self.map.get(kana).cloned()
    }
}
