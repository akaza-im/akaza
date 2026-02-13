use std::collections::HashMap;

use anyhow::Result;
use log::info;

use libakaza::cost::calc_cost;
use libakaza::lm::base::SystemSkipBigramLM;
use rsmarisa::{Agent, Keyset, Trie};

/// skip-bigram 言語モデル（カウントベース）のビルダー。
/// キー形式: [3B id1][3B id2][4B u32_count]
pub struct WordcntSkipBigramBuilder {
    keyset: Keyset,
}

impl Default for WordcntSkipBigramBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
        }
    }
}

impl WordcntSkipBigramBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, cnt: u32) {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let mut key: Vec<u8> = Vec::new();
        key.extend(id1_bytes[0..3].iter());
        key.extend(id2_bytes[0..3].iter());
        key.extend(cnt.to_le_bytes());
        self.keyset.push_back_bytes(&key, 1.0).unwrap();
    }

    pub fn save(&mut self, ofname: &str) -> anyhow::Result<()> {
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        trie.save(ofname)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub struct WordcntSkipBigram {
    trie: Trie,
    pub total_words: u32,
    pub unique_words: u32,
}

#[allow(dead_code)]
impl WordcntSkipBigram {
    pub fn load(filename: &str) -> Result<WordcntSkipBigram> {
        info!("Loading system-skip-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;

        let map = Self::to_cnt_map_inner(&trie);
        let total_words = map.iter().map(|((_, _), cnt)| *cnt).sum();
        let unique_words = map.keys().count() as u32;

        Ok(WordcntSkipBigram {
            trie,
            total_words,
            unique_words,
        })
    }

    pub fn to_cnt_map(&self) -> HashMap<(i32, i32), u32> {
        Self::to_cnt_map_inner(&self.trie)
    }

    fn to_cnt_map_inner(trie: &Trie) -> HashMap<(i32, i32), u32> {
        let mut map: HashMap<(i32, i32), u32> = HashMap::new();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            if word.len() == 10 {
                let word_id1 = i32::from_le_bytes([word[0], word[1], word[2], 0]);
                let word_id2 = i32::from_le_bytes([word[3], word[4], word[5], 0]);
                let cnt = u32::from_le_bytes([word[6], word[7], word[8], word[9]]);
                map.insert((word_id1, word_id2), cnt);
            }
        }
        map
    }
}

impl SystemSkipBigramLM for WordcntSkipBigram {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes()[0..3].iter());
        key.extend(word_id2.to_le_bytes()[0..3].iter());

        let mut agent = Agent::new();
        agent.set_query_bytes(&key);

        if self.trie.predictive_search(&mut agent) {
            let keyword = agent.key().as_bytes();
            let last4: [u8; 4] = keyword[keyword.len() - 4..keyword.len()]
                .try_into()
                .unwrap();
            let score: u32 = u32::from_le_bytes(last4);
            return Some(calc_cost(score, self.total_words, self.unique_words));
        }

        None
    }

    fn get_default_skip_cost(&self) -> f32 {
        calc_cost(0, self.total_words, self.unique_words)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_build() -> Result<()> {
        let named_tmpfile = NamedTempFile::new().unwrap();
        let tmpfile = named_tmpfile.path().to_str().unwrap().to_string();

        let mut builder = WordcntSkipBigramBuilder::default();
        builder.add(4, 5, 29);
        builder.add(8, 9, 32);
        builder.save(tmpfile.as_str())?;

        let skip_bigram = WordcntSkipBigram::load(tmpfile.as_str())?;
        assert_eq!(
            skip_bigram.to_cnt_map(),
            HashMap::from([((4, 5), 29), ((8, 9), 32),])
        );

        Ok(())
    }
}
