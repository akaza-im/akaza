use crawdad::Trie;

use crate::kana_trie::base::KanaTrie;

struct CrawdadKanaTrie {
    trie: Trie,
}

impl CrawdadKanaTrie {
    pub fn build(keys: Vec<String>) -> anyhow::Result<CrawdadKanaTrie> {
        let trie = Trie::from_keys(keys).unwrap();
        Ok(CrawdadKanaTrie { trie })
    }
}

impl KanaTrie for CrawdadKanaTrie {
    fn common_prefix_search(&self, query: &str) -> Vec<String> {
        // もう少しスマートに書けそう。
        let mut p = Vec::new();
        let haystack: Vec<char> = query.chars().collect();
        for (_, s) in self.trie.common_prefix_search(haystack.iter().copied()) {
            let (k, _) = query.char_indices().nth(s).unwrap();
            p.push(query[0..k].to_string())
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello() -> anyhow::Result<()> {
        let trie = CrawdadKanaTrie::build(vec![
            "わたし".to_string(),
            "わた".to_string(),
            "わし".to_string(),
            "ほげほげ".to_string(),
        ])?;
        assert_eq!(
            trie.common_prefix_search("わたしのきもち"),
            vec!("わた", "わたし")
        );
        Ok(())
    }
}
