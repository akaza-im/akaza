use crate::trie::{Trie, TrieBuilder};

/**
 * バイナリ辞書。
 *
 * バイナリ辞書はシステム辞書に利用されている。
 * 「よみ」から「漢字」への変換辞書である。
 *
 * なぜバイナリ辞書と呼ばれているかは、歴史的経緯と言わざるを得ない。
 * TODO: これはいずれ KanaKanjiDict とでも改名すべきであろう。
 * rust 化が済んだあとに。。
 */

pub struct KanaKanjiDictBuilder {
    trie_builder: TrieBuilder,
}

impl KanaKanjiDictBuilder {
    pub unsafe fn new() -> KanaKanjiDictBuilder {
        KanaKanjiDictBuilder {
            trie_builder: TrieBuilder::new(),
        }
    }
    pub unsafe fn add(&self, yomi: &String, kanjis: &String) {
        let key = [yomi.as_bytes(), b"\t", kanjis.as_bytes()].concat();
        self.trie_builder.add(key);
    }

    pub unsafe fn save(&self, filename: &String) -> std::io::Result<()> {
        return self.trie_builder.save(filename);
    }
}

pub struct KanaKanjiDict {
    trie: Trie,
}
impl KanaKanjiDict {
    pub fn load(file_name: &String) -> Result<KanaKanjiDict, String> {
        unsafe {
            match Trie::load(file_name) {
                Ok(trie) => Ok(KanaKanjiDict { trie }),
                Err(err) => Err(err.to_string()),
            }
        }
    }
    /*
       pub fn find(&self, yomi: &String) {
           unsafe {
               let got = self
                   .trie
                   .predictive_search([yomi.as_bytes(), b"\t"].concat().to_vec());
               for result in got {
                   let p = String::from_utf8(result.keyword)
                       .unwrap()
                       .splitn(2, "\t")
                       .collect();
                   let yomi = p[0];
                   let kanjis = p[1];
               }
           }
       }
    */

    pub fn all_yomis(&self) -> Vec<String> {
        unsafe {
            let mut result: Vec<String> = Vec::new();
            let got = self.trie.predictive_search(b"".to_vec());
            for item in got {
                let item: String = String::from_utf8(item.keyword).unwrap();
                let p: Vec<&str> = item.splitn(2, "\t").collect();
                let yomi: &str = p[0];
                result.push(yomi.to_string())
            }
            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let tmpfile = "/tmp/kanakanji.tri".to_string();
        unsafe {
            let builder = KanaKanjiDictBuilder::new();
            builder.add(&"わたし".to_string(), &"私/渡し".to_string());
            builder.add(&"なまえ".to_string(), &"名前".to_string());
            builder.save(&tmpfile).unwrap();
        }

        {
            let dict = KanaKanjiDict::load(&tmpfile).unwrap();
            let mut yomis = dict.all_yomis();
            yomis.sort();
            assert_eq!(yomis, vec!["なまえ".to_string(), "わたし".to_string(),]);
        }
    }
}
