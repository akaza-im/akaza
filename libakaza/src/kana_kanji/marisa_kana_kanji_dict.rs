use std::collections::HashMap;

use log::trace;

use marisa_sys::{Keyset, Marisa};

use crate::kana_kanji::base::KanaKanjiDict;

#[derive(Default)]
pub struct MarisaKanaKanjiDict {
    marisa: Marisa,
}

impl MarisaKanaKanjiDict {
    pub(crate) fn build(
        dicts: HashMap<String, Vec<String>>,
        cache_path: &str,
        cache_serialized_key: &str,
    ) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut keyset = Keyset::default();
        for (kana, surfaces) in dicts {
            keyset.push_back(
                [
                    kana.as_bytes(),
                    b"\t", // seperator
                    surfaces.join("/").as_bytes(),
                ]
                .concat()
                .as_slice(),
            );
        }
        keyset.push_back(
            [
                "__CACHE_SERIALIZED__\t".as_bytes(),
                cache_serialized_key.as_bytes(),
            ]
            .concat()
            .as_slice(),
        );

        let mut marisa = Marisa::default();
        marisa.build(&keyset);
        marisa.save(cache_path)?;
        Ok(MarisaKanaKanjiDict { marisa })
    }

    pub fn load(file_name: &str) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut marisa = Marisa::default();
        marisa.load(file_name)?;
        Ok(MarisaKanaKanjiDict { marisa })
    }

    pub fn cache_serialized(&self) -> String {
        let mut p = String::new();
        self.marisa
            .predictive_search("__CACHE_SERIALIZED__\t".as_bytes(), |word, _| {
                let idx = word.iter().position(|f| *f == b'\t').unwrap();
                p = String::from_utf8_lossy(&word[idx + 1..word.len()]).to_string();
                false
            });
        p
    }

    pub fn yomis(&self) -> Vec<String> {
        let mut yomis: Vec<String> = Vec::new();

        self.marisa.predictive_search("".as_bytes(), |word, _| {
            if !word.starts_with("__CACHE_SERIALIZED__\t".as_bytes()) {
                let idx = word.iter().position(|f| *f == b'\t').unwrap();
                yomis.push(String::from_utf8_lossy(&word[0..idx]).to_string());
            }
            true
        });

        yomis
    }
}

impl KanaKanjiDict for MarisaKanaKanjiDict {
    fn get(&self, kana: &str) -> Option<Vec<String>> {
        let mut surfaces: Vec<String> = Vec::new();
        let query = [kana.as_bytes(), b"\t".as_slice()].concat();
        self.marisa.predictive_search(query.as_slice(), |word, _| {
            let idx = word.iter().position(|f| *f == b'\t').unwrap();
            let s = String::from_utf8_lossy(&word[idx + 1..word.len()]).to_string();
            for s in s.split('/').collect::<Vec<_>>() {
                surfaces.push(s.to_string());
            }
            false
        });
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

        let dict = MarisaKanaKanjiDict::build(
            HashMap::from([("たなか".to_string(), vec!["田中".to_string()])]),
            path.as_str(),
            "",
        )?;

        assert_eq!(dict.get("たなか"), Some(vec!["田中".to_string()]));

        Ok(())
    }
}
