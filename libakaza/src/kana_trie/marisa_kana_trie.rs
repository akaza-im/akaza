use std::cell::RefCell;

use anyhow::Result;
use log::info;

use rsmarisa::{Agent, Keyset, Trie};

use crate::kana_trie::base::KanaTrie;

/// marisa-trie ベースの kana_trie。
///
/// 用途は `Segmenter` での共通接頭辞探索（読みのセグメント候補列挙）のみ。
/// 起動時に毎回 cedarwood を 100 万件の `update()` で構築すると 1 秒前後かかるため、
/// 同じ読み集合から marisa-trie を一度ビルドして cache file に保存し、
/// 次回以降は load だけで済むようにしている。
pub struct MarisaKanaTrie {
    trie: Trie,
    /// 検索ごとに `Agent::new()` するアロケーションを避けるため再利用する。
    agent: RefCell<Agent>,
}

impl MarisaKanaTrie {
    pub fn build_and_save<I, S>(keys: I, path: &str) -> Result<MarisaKanaTrie>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut keyset = Keyset::new();
        for k in keys {
            let s = k.as_ref();
            if s.is_empty() {
                continue;
            }
            keyset.push_back_str(s)?;
        }
        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        trie.save(path)?;
        info!("Saved kana_trie cache: {}", path);
        Ok(MarisaKanaTrie {
            trie,
            agent: RefCell::new(Agent::new()),
        })
    }

    pub fn load(path: &str) -> Result<MarisaKanaTrie> {
        let mut trie = Trie::new();
        trie.load(path)?;
        Ok(MarisaKanaTrie {
            trie,
            agent: RefCell::new(Agent::new()),
        })
    }
}

impl KanaTrie for MarisaKanaTrie {
    fn common_prefix_search(&self, query: &str) -> Vec<String> {
        // CedarwoodKanaTrie 同様、query の prefix と一致する登録済み読みを全部返す。
        // marisa の common_prefix_search は呼ぶたびに次のヒットを返す iterator 仕様。
        let mut agent = self.agent.borrow_mut();
        agent.set_query_str(query);

        let mut out: Vec<String> = Vec::new();
        while self.trie.common_prefix_search(&mut agent) {
            let bytes = agent.key().as_bytes();
            // UTF-8 として有効。yomi のみを格納しているので tab などは含まれない。
            match std::str::from_utf8(bytes) {
                Ok(s) => out.push(s.to_string()),
                Err(_) => continue,
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn build_and_search() -> Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_str().unwrap().to_string();

        let trie =
            MarisaKanaTrie::build_and_save(vec!["わたし", "わた", "わし", "ほげほげ"], &path)?;

        let mut got = trie.common_prefix_search("わたしのきもち");
        got.sort();
        assert_eq!(got, vec!["わた".to_string(), "わたし".to_string()]);
        Ok(())
    }

    #[test]
    fn save_load_roundtrip() -> Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_str().unwrap().to_string();

        MarisaKanaTrie::build_and_save(vec!["abc", "ab", "abcd"], &path)?;
        let loaded = MarisaKanaTrie::load(&path)?;
        let mut got = loaded.common_prefix_search("abcde");
        got.sort();
        assert_eq!(got, vec!["ab", "abc", "abcd"]);
        Ok(())
    }

    #[test]
    fn empty_input_ignored() -> Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_str().unwrap().to_string();
        let trie = MarisaKanaTrie::build_and_save(vec!["abc", ""], &path)?;
        let got = trie.common_prefix_search("abc");
        assert_eq!(got, vec!["abc".to_string()]);
        Ok(())
    }
}
