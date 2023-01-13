use anyhow::Result;

use marisa_sys::{Keyset, Marisa};

use crate::trie::SearchResult;

/**
 * 「よみ」から「漢字」への変換辞書である。
 *
 * 空間効率が良いので現在の実装では MARISA を利用している。
 * 実際には普通にキーバリューストアとしてしか利用していない。
 * 別のデータ構造もそのうち検討したほうが良いかもしれない。
 */
#[derive(Default)]
pub struct KanaKanjiDictBuilder {
    keyset: Keyset,
}

impl KanaKanjiDictBuilder {
    pub fn add(&mut self, yomi: &str, kanjis: &str) -> &mut KanaKanjiDictBuilder {
        let key = [yomi.as_bytes(), b"\t", kanjis.as_bytes()].concat();
        self.keyset.push_back(key.as_slice());
        self
    }

    pub fn save(&self, filename: &str) -> Result<()> {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        marisa.save(filename)?;
        Ok(())
    }

    pub fn build(&self) -> KanaKanjiDict {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        KanaKanjiDict { marisa }
    }
}

pub struct KanaKanjiDict {
    marisa: Marisa,
}

impl Default for KanaKanjiDict {
    fn default() -> Self {
        KanaKanjiDictBuilder::default().build()
    }
}

impl KanaKanjiDict {
    pub fn load(file_name: &str) -> Result<KanaKanjiDict> {
        let mut marisa = Marisa::default();
        marisa.load(file_name)?;
        Ok(KanaKanjiDict { marisa })
    }

    pub fn find(&self, yomi: &str) -> Option<Vec<String>> {
        let keyword = [yomi.as_bytes(), b"\t"].concat().to_vec();
        let mut got: Vec<SearchResult> = Vec::new();
        self.marisa
            .predictive_search(keyword.as_slice(), |key, id| {
                got.push(SearchResult {
                    keyword: key.to_vec(),
                    id,
                });
                true
            });
        if let Some(result) = got.into_iter().next() {
            let s: String = String::from_utf8(result.keyword).unwrap();
            let (_, kanjis) = s.split_once('\t')?;
            return Some(kanjis.split('/').map(|f| f.to_string()).collect());
        }
        None
    }

    pub fn all_yomis(&self) -> Option<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        let keyword = b"".to_vec();
        let mut got: Vec<SearchResult> = Vec::new();
        self.marisa
            .predictive_search(keyword.as_slice(), |key, id| {
                got.push(SearchResult {
                    keyword: key.to_vec(),
                    id,
                });
                true
            });
        for item in got {
            let item: String = String::from_utf8(item.keyword).unwrap();
            let (yomi, _) = item.split_once('\t')?;
            result.push(yomi.to_string())
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_yomis() {
        let tmpfile = "/tmp/kanakanji.tri".to_string();
        {
            let mut builder = KanaKanjiDictBuilder::default();
            builder.add("わたし", "私/渡し");
            builder.add("なまえ", "名前");
            builder.save(&tmpfile).unwrap();
        }

        {
            let dict = KanaKanjiDict::load(&tmpfile).unwrap();
            let mut yomis = dict.all_yomis().unwrap();
            yomis.sort();
            assert_eq!(yomis, vec!["なまえ".to_string(), "わたし".to_string()]);
        }
    }

    #[test]
    fn test_find() {
        let mut builder = KanaKanjiDictBuilder::default();
        builder.add("わたし", "私/渡し");
        builder.add("なまえ", "名前");
        let dict = builder.build();

        let mut kanjis = dict.find("わたし").unwrap();
        kanjis.sort();
        assert_eq!(kanjis, vec!["渡し".to_string(), "私".to_string()]);
    }
}
