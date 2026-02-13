use std::collections::HashMap;

use log::trace;

use rsmarisa::{Agent, Keyset, Trie};

use crate::kana_kanji::base::KanaKanjiDict;

pub struct MarisaKanaKanjiDict {
    trie: Trie,
}

impl MarisaKanaKanjiDict {
    pub(crate) fn build_with_cache(
        dict: HashMap<String, Vec<String>>,
        cache_path: &str,
        cache_serialized_key: &str,
    ) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut keyset = Self::build_keyset(dict);
        let cache_key = format!("__CACHE_SERIALIZED__\t{}", cache_serialized_key);
        keyset.push_back_str(&cache_key)?;

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        trie.save(cache_path)?;
        Ok(MarisaKanaKanjiDict { trie })
    }

    pub(crate) fn build(dict: HashMap<String, Vec<String>>) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut keyset = Self::build_keyset(dict);
        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        Ok(MarisaKanaKanjiDict { trie })
    }

    pub fn build_keyset(dict: HashMap<String, Vec<String>>) -> Keyset {
        let mut keyset = Keyset::new();
        for (kana, surfaces) in dict {
            let entry = format!("{}\t{}", kana, surfaces.join("/"));
            keyset.push_back_str(&entry).unwrap();
        }
        keyset
    }

    pub fn load(file_name: &str) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut trie = Trie::new();
        trie.load(file_name)?;
        Ok(MarisaKanaKanjiDict { trie })
    }

    pub fn cache_serialized(&self) -> String {
        let mut agent = Agent::new();
        agent.set_query_str("__CACHE_SERIALIZED__\t");

        if self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            if let Some(idx) = word.iter().position(|f| *f == b'\t') {
                return String::from_utf8_lossy(&word[idx + 1..]).to_string();
            }
        }
        String::new()
    }

    pub fn yomis(&self) -> Vec<String> {
        let mut yomis: Vec<String> = Vec::new();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            if !word.starts_with(b"__CACHE_SERIALIZED__\t") {
                if let Some(idx) = word.iter().position(|f| *f == b'\t') {
                    yomis.push(String::from_utf8_lossy(&word[0..idx]).to_string());
                }
            }
        }

        yomis
    }
}

impl KanaKanjiDict for MarisaKanaKanjiDict {
    fn get(&self, kana: &str) -> Option<Vec<String>> {
        let mut surfaces: Vec<String> = Vec::new();
        let query = format!("{}\t", kana);
        let mut agent = Agent::new();
        agent.set_query_str(&query);

        if self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            if let Some(idx) = word.iter().position(|f| *f == b'\t') {
                let s = String::from_utf8_lossy(&word[idx + 1..]).to_string();
                for s in s.split('/') {
                    surfaces.push(s.to_string());
                }
            }
        }

        trace!("Got result: {:?}, {:?}", kana, surfaces);
        Some(surfaces)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn write_read() -> anyhow::Result<()> {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let dict = MarisaKanaKanjiDict::build_with_cache(
            HashMap::from([("たなか".to_string(), vec!["田中".to_string()])]),
            path.as_str(),
            "",
        )?;

        assert_eq!(dict.get("たなか"), Some(vec!["田中".to_string()]));

        Ok(())
    }
}
