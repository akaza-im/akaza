use std::collections::HashMap;

use anyhow::Result;
use log::info;

use libakaza::cost::calc_cost;
use libakaza::lm::base::SystemUnigramLM;
use rsmarisa::{Agent, Keyset, Trie};

/**
 * unigram 言語モデル。
 * 「漢字/かな」に対して、発生確率スコアを保持している。
 */
#[derive(Default)]
pub struct WordcntUnigramBuilder {
    data: Vec<(String, u32)>,
}

impl WordcntUnigramBuilder {
    pub fn add(&mut self, word: &str, cnt: u32) {
        self.data.push((word.to_string(), cnt));
    }

    pub fn keyset(&self) -> Result<Keyset> {
        let mut keyset = Keyset::new();
        for (kanji, score) in &self.data {
            // 区切り文字をいれなくても、末尾の4バイトを取り出せば十分な気がしないでもない。。
            // 先頭一致にして、+4バイトになるものを探せばいいはず。
            // 最適化の余地だけど、現実的には空間効率よりも速度のほうが重要かもしれない。
            let key = [
                kanji.as_bytes(),
                b"\xff",
                score.to_le_bytes().as_slice(), // バイナリにしてデータ容量を節約する
            ]
            .concat();
            keyset.push_back_bytes(&key, 1.0)?;
        }
        Ok(keyset)
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        let mut keyset = self.keyset()?;
        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        trie.save(fname)?;
        Ok(())
    }
}

pub struct WordcntUnigram {
    trie: Trie,
    pub(crate) total_words: u32,
    pub(crate) unique_words: u32,
}

impl WordcntUnigram {
    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    pub fn to_count_hashmap(&self) -> HashMap<String, (i32, u32)> {
        Self::_to_count_hashmap(&self.trie)
    }

    fn _to_count_hashmap(trie: &Trie) -> HashMap<String, (i32, u32)> {
        let mut map: HashMap<String, (i32, u32)> = HashMap::new();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            let id = agent.key().id();

            if let Some(idx) = word.iter().position(|f| *f == b'\xff') {
                let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
                let word_str = String::from_utf8_lossy(&word[0..idx]);
                let cost = u32::from_le_bytes(bytes);
                map.insert(word_str.to_string(), (id as i32, cost));
            }
        }
        map
    }

    pub fn load(fname: &str) -> Result<WordcntUnigram> {
        info!("Reading {}", fname);
        let mut trie = Trie::new();
        trie.load(fname)?;

        let map = Self::_to_count_hashmap(&trie);

        // 総出現単語数
        let total_words = map.iter().map(|(_, (_, cnt))| *cnt).sum();
        // 単語の種類数
        let unique_words = map.keys().count() as u32;

        Ok(WordcntUnigram {
            trie,
            total_words,
            unique_words,
        })
    }
}

impl SystemUnigramLM for WordcntUnigram {
    fn get_cost(&self, wordcnt: u32) -> f32 {
        calc_cost(wordcnt, self.total_words, self.unique_words)
    }

    /// @return (word_id, score)。
    fn find(&self, word: &str) -> Option<(i32, f32)> {
        assert_ne!(word.len(), 0);

        let mut key = word.as_bytes().to_vec();
        key.push(0xff);
        let mut agent = Agent::new();
        agent.set_query_bytes(&key);

        if self.trie.predictive_search(&mut agent) {
            let word_bytes = agent.key().as_bytes();
            let word_id = agent.key().id();

            if let Some(idx) = word_bytes.iter().position(|f| *f == b'\xff') {
                let bytes: [u8; 4] = word_bytes[idx + 1..idx + 1 + 4].try_into().unwrap();
                let score = u32::from_le_bytes(bytes);
                return Some((
                    word_id as i32,
                    calc_cost(score, self.total_words, self.unique_words),
                ));
            }
        }

        None
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        let mut map = HashMap::new();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            let id = agent.key().id();

            if let Some(idx) = word.iter().position(|f| *f == b'\xff') {
                let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
                let word_str = String::from_utf8_lossy(&word[0..idx]);
                let cnt = u32::from_le_bytes(bytes);
                map.insert(
                    word_str.to_string(),
                    (
                        id as i32,
                        calc_cost(cnt, self.total_words, self.unique_words),
                    ),
                );
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test() -> Result<()> {
        let named_tmpfile = NamedTempFile::new().unwrap();
        let tmpfile = named_tmpfile.path().to_str().unwrap().to_string();

        let mut builder = WordcntUnigramBuilder::default();
        builder.add("私/わたし", 3);
        builder.add("彼/かれ", 42);
        builder.save(tmpfile.as_str())?;

        let wordcnt = WordcntUnigram::load(tmpfile.as_str())?;
        assert_eq!(
            wordcnt.to_count_hashmap(),
            HashMap::from([
                ("私/わたし".to_string(), (1_i32, 3_u32)),
                ("彼/かれ".to_string(), (0_i32, 42_u32)),
            ])
        );
        assert_eq!(wordcnt.total_words, 45); // 単語発生数
        assert_eq!(wordcnt.unique_words, 2); // ユニーク単語数
        assert_eq!(wordcnt.get_cost(0), 6.672098);
        assert_eq!(wordcnt.get_cost(1), 1.6720936);

        assert_eq!(wordcnt.find("私/わたし"), Some((1_i32, 1.1949753)));
        assert_eq!(wordcnt.find("彼/かれ"), Some((0_i32, 0.048848562)));

        assert_eq!(
            wordcnt.as_hash_map(),
            HashMap::from([
                ("私/わたし".to_string(), (1_i32, 1.1949753)),
                ("彼/かれ".to_string(), (0_i32, 0.048848562)),
            ])
        );

        Ok(())
    }
}
