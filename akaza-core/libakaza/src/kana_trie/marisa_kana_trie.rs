use anyhow::Result;

use crate::kana_trie::base::KanaTrie;
use marisa_sys::{Keyset, Marisa};

#[derive(Default)]
pub struct MarisaKanaTrie {
    marisa: Marisa,
}

impl MarisaKanaTrie {
    pub fn build(keys: Vec<String>) -> MarisaKanaTrie {
        let mut keyset = Keyset::default();
        for key in keys {
            keyset.push_back(key.as_bytes());
        }

        let mut marisa = Marisa::default();
        marisa.build(&keyset);
        MarisaKanaTrie { marisa }
    }

    pub(crate) fn save(&self, file_name: &str) -> Result<()> {
        self.marisa.save(file_name)
    }

    pub fn load(file_name: &str) -> Result<MarisaKanaTrie> {
        let mut marisa = Marisa::default();
        marisa.load(file_name)?;
        Ok(MarisaKanaTrie { marisa })
    }
}
impl KanaTrie for MarisaKanaTrie {
    fn common_prefix_search(&self, query: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        self.marisa.common_prefix_search(query, |word, _| {
            result.push(String::from_utf8(word.to_vec()).unwrap());
            true
        });
        result
    }
}
