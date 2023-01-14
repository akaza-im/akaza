use cedarwood::Cedar;

use crate::kana_trie::base::KanaTrie;

pub struct CedarwoodKanaTrie {
    cedar: Cedar,
    words: Vec<String>,
}

impl Default for CedarwoodKanaTrie {
    fn default() -> Self {
        let cedar = Cedar::new();
        CedarwoodKanaTrie {
            cedar,
            words: Vec::new(),
        }
    }
}

impl CedarwoodKanaTrie {
    pub fn build(keys: Vec<String>) -> CedarwoodKanaTrie {
        let mut cedar = Cedar::new();
        let mut words: Vec<String> = Vec::new();
        for key in keys {
            cedar.update(key.as_str(), words.len() as i32);
            words.push(key);
        }
        CedarwoodKanaTrie { cedar, words }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cedar.exact_match_search(key).is_some()
    }

    pub fn update(&mut self, key: &str) {
        self.cedar.update(key, self.words.len() as i32);
        self.words.push(key.to_string());
    }
}

impl KanaTrie for CedarwoodKanaTrie {
    fn common_prefix_search(&self, query: &str) -> Vec<String> {
        self.cedar
            .common_prefix_iter(query)
            .map(|(n, _)| self.words[n as usize].clone())
            .collect::<Vec<String>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello() -> anyhow::Result<()> {
        let trie = CedarwoodKanaTrie::build(vec![
            "わたし".to_string(),
            "わた".to_string(),
            "わし".to_string(),
            "ほげほげ".to_string(),
        ]);
        assert_eq!(
            trie.common_prefix_search("わたしのきもち"),
            vec!("わた", "わたし")
        );
        Ok(())
    }
}
