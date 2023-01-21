use marisa_sys::Marisa;

use crate::kana_kanji::base::KanaKanjiDict;

#[derive(Default)]
pub struct MarisaKanaKanjiDict {
    marisa: Marisa,
}

impl MarisaKanaKanjiDict {
    pub(crate) fn new(marisa: Marisa) -> Self {
        MarisaKanaKanjiDict { marisa }
    }

    pub fn load(file_name: &str) -> anyhow::Result<MarisaKanaKanjiDict> {
        let mut marisa = Marisa::default();
        marisa.load(file_name)?;
        Ok(MarisaKanaKanjiDict { marisa })
    }

    pub fn yomis(&self) -> Vec<String> {
        let mut yomis: Vec<String> = Vec::new();

        self.marisa.predictive_search("".as_bytes(), |word, _| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            yomis.push(String::from_utf8_lossy(&&word[0..idx]).to_string());
            true
        });

        yomis
    }
}

impl KanaKanjiDict for MarisaKanaKanjiDict {
    fn get(&self, kana: &str) -> Option<Vec<String>> {
        let mut surfaces: Vec<String> = Vec::new();
        let query = [kana.as_bytes(), b"\xff".as_slice()].concat();
        self.marisa.predictive_search(query.as_slice(), |word, _| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let s = String::from_utf8_lossy(&&word[0..idx]).to_string();
            for s in s.split('/').collect::<Vec<_>>() {
                surfaces.push(s.to_string());
            }
            false
        });
        Some(surfaces)
    }
}
