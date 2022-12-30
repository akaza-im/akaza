use crate::trie::{Trie, TrieBuilder};

/**
 * 「よみ」から「漢字」への変換辞書である。
 *
 * 空間効率が良いので現在の実装では MARISA を利用している。
 * 実際には普通にキーバリューストアとしてしか利用していない。
 * 別のデータ構造もそのうち検討したほうが良いかもしれない。
 */

pub struct KanaKanjiDictBuilder {
    trie_builder: TrieBuilder,
}

impl KanaKanjiDictBuilder {
    pub fn new() -> KanaKanjiDictBuilder {
        KanaKanjiDictBuilder {
            trie_builder: TrieBuilder::new(),
        }
    }

    pub fn add(&self, yomi: &String, kanjis: &String) {
        let key = [yomi.as_bytes(), b"\t", kanjis.as_bytes()].concat();
        self.trie_builder.add(key);
    }

    pub fn save(&self, filename: &String) -> std::io::Result<()> {
        return self.trie_builder.save(filename);
    }

    pub fn build(&self) -> KanaKanjiDict {
        KanaKanjiDict {
            trie: self.trie_builder.build(),
        }
    }
}

pub struct KanaKanjiDict {
    trie: Trie,
}

impl KanaKanjiDict {
    pub fn load(file_name: &String) -> Result<KanaKanjiDict, String> {
        match Trie::load(file_name) {
            Ok(trie) => Ok(KanaKanjiDict { trie }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn find(&self, yomi: &String) -> Option<Vec<String>> {
        let got = self
            .trie
            .predictive_search([yomi.as_bytes(), b"\t"].concat().to_vec());
        for result in got {
            let s: String = String::from_utf8(result.keyword).unwrap();
            let (_, kanjis) = s.split_once("\t")?;
            return Some(kanjis.split("/").map(|f| f.to_string()).collect());
        }
        return None;
    }

    pub fn all_yomis(&self) -> Option<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        let got = self.trie.predictive_search(b"".to_vec());
        for item in got {
            let item: String = String::from_utf8(item.keyword).unwrap();
            let (yomi, _) = item.split_once("\t")?;
            result.push(yomi.to_string())
        }
        return Some(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_yomis() {
        let tmpfile = "/tmp/kanakanji.tri".to_string();
        {
            let builder = KanaKanjiDictBuilder::new();
            builder.add(&"わたし".to_string(), &"私/渡し".to_string());
            builder.add(&"なまえ".to_string(), &"名前".to_string());
            builder.save(&tmpfile).unwrap();
        }

        {
            let dict = KanaKanjiDict::load(&tmpfile).unwrap();
            let mut yomis = dict.all_yomis().unwrap();
            yomis.sort();
            assert_eq!(yomis, vec!["なまえ".to_string(), "わたし".to_string(),]);
        }
    }

    #[test]
    fn test_find() {
        let builder = KanaKanjiDictBuilder::new();
        builder.add(&"わたし".to_string(), &"私/渡し".to_string());
        builder.add(&"なまえ".to_string(), &"名前".to_string());
        let dict = builder.build();

        let mut kanjis = dict.find(&"わたし".to_string()).unwrap();
        kanjis.sort();
        assert_eq!(kanjis, vec!["渡し".to_string(), "私".to_string(),]);
    }
}
