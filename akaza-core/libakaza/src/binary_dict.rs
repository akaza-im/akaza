use crate::trie::TrieBuilder;

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

pub struct BinaryDict {
    trie_builder: TrieBuilder,
}

impl BinaryDict {
    pub unsafe fn new() -> BinaryDict {
        BinaryDict {
            trie_builder: TrieBuilder::new(),
        }
    }
    pub unsafe fn add(&self, yomi: &String, kanjis: &String) {
        let key = [yomi.as_bytes(), b"\xff", kanjis.as_bytes()].concat();
        self.trie_builder.add(key);
    }

    pub unsafe fn save(&self, filename: &String) -> std::io::Result<()> {
        return self.trie_builder.save(filename);
    }
}
