use std::collections::HashMap;

use anyhow::Result;
use log::info;

use libakaza::cost::calc_cost;
use libakaza::lm::base::SystemBigramLM;
use libakaza::trie::SearchResult;
use marisa_sys::{Keyset, Marisa};

/**
 * bigram 言語モデル。
 * unigram の生成のときに得られた単語IDを利用することで、圧縮している。
 */
#[derive(Default)]
pub struct WordcntBigramBuilder {
    keyset: Keyset,
}

impl WordcntBigramBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, cnt: u32) {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let mut key: Vec<u8> = Vec::new();
        key.extend(id1_bytes[0..3].iter());
        key.extend(id2_bytes[0..3].iter());
        key.extend(cnt.to_le_bytes());
        self.keyset.push_back(key.as_slice());
    }

    pub fn save(&self, ofname: &str) -> anyhow::Result<()> {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        marisa.save(ofname)?;
        Ok(())
    }
}

pub struct WordcntBigram {
    marisa: Marisa,
    default_edge_cost: f32,
    pub c: u32,
    pub v: u32,
}

impl WordcntBigram {
    pub fn to_cnt_map(&self) -> HashMap<(i32, i32), u32> {
        Self::_to_map(&self.marisa)
    }

    fn _to_map(marisa: &Marisa) -> HashMap<(i32, i32), u32> {
        let mut map: HashMap<(i32, i32), u32> = HashMap::new();
        marisa.predictive_search("".as_bytes(), |word, _id| {
            if word.len() == 8 {
                let word_id1 = i32::from_le_bytes([word[0], word[1], word[2], 0]);
                let word_id2 = i32::from_le_bytes([word[3], word[4], word[5], 0]);
                let cost = u32::from_le_bytes([word[6], word[7], word[8], word[9]]);
                map.insert((word_id1, word_id2), cost);
            }
            true
        });
        map
    }

    pub fn load(filename: &str) -> Result<WordcntBigram> {
        info!("Loading system-bigram: {}", filename);
        let mut marisa = Marisa::default();
        marisa.load(filename)?;

        let map: HashMap<(i32, i32), u32> = Self::_to_map(&marisa);

        // 総出現単語数
        let c = map.iter().map(|((_, _), cnt)| *cnt).sum();
        // 単語の種類数
        let v = map.keys().count() as u32;
        let default_edge_cost = calc_cost(0, c, v);

        Ok(WordcntBigram {
            marisa,
            default_edge_cost,
            c,
            v,
        })
    }
}

impl SystemBigramLM for WordcntBigram {
    fn get_default_edge_cost(&self) -> f32 {
        self.default_edge_cost
    }

    /**
     * edge cost を得る。
     * この ID は、unigram の trie でふられたもの。
     */
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes()[0..3].iter());
        key.extend(word_id2.to_le_bytes()[0..3].iter());
        let mut got: Vec<SearchResult> = Vec::new();
        self.marisa.predictive_search(key.as_slice(), |key, id| {
            got.push(SearchResult {
                keyword: key.to_vec(),
                id,
            });
            true
        });
        let Some(result) = got.first() else {
            return None;
        };
        let last2: [u8; 4] = result.keyword[result.keyword.len() - 4..result.keyword.len()]
            .try_into()
            .unwrap();
        let score: u32 = u32::from_le_bytes(last2);
        Some(calc_cost(score, self.c, self.v))
    }

    fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        let mut map: HashMap<(i32, i32), f32> = HashMap::new();
        self.marisa.predictive_search("".as_bytes(), |word, _id| {
            if word.len() == 8 {
                let word_id1 = i32::from_le_bytes([word[0], word[1], word[2], 0]);
                let word_id2 = i32::from_le_bytes([word[3], word[4], word[5], 0]);
                let cnt = u32::from_le_bytes([word[6], word[7], word[8], word[9]]);
                map.insert((word_id1, word_id2), calc_cost(cnt, self.c, self.v));
            }
            true
        });
        map
    }
}
