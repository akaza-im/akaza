use std::collections::HashMap;

use anyhow::Result;
use log::info;

use libakaza::cost::calc_cost;
use libakaza::lm::base::SystemUnigramLM;
use marisa_sys::{Keyset, Marisa};

/**
 * unigram 言語モデル。
 * 「漢字/かな」に対して、発生確率スコアを保持している。
 */
#[derive(Default)]
pub struct WordcntUnigramBuilder {
    data: Vec<(String, u32)>,
}

impl WordcntUnigramBuilder {
    pub fn add(&mut self, word: &str, score: u32) {
        self.data.push((word.to_string(), score));
    }

    pub fn keyset(&self) -> Keyset {
        let mut keyset = Keyset::default();
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
            keyset.push_back(key.as_slice());
        }
        keyset
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset());
        marisa.save(fname)?;
        Ok(())
    }
}

pub struct WordcntUnigram {
    marisa: Marisa,
    default_cost: f32,
    default_cost_for_short: f32,
    pub(crate) c: u32,
    pub(crate) v: u32,
}

impl WordcntUnigram {
    pub fn num_keys(&self) -> usize {
        self.marisa.num_keys()
    }

    pub fn to_count_hashmap(&self) -> HashMap<String, (i32, u32)> {
        Self::_to_count_hashmap(&self.marisa)
    }

    fn _to_count_hashmap(marisa: &Marisa) -> HashMap<String, (i32, u32)> {
        let mut map: HashMap<String, (i32, u32)> = HashMap::new();
        marisa.predictive_search("".as_bytes(), |word, id| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            let word = String::from_utf8_lossy(&word[0..idx]);
            let cost = u32::from_le_bytes(bytes);
            map.insert(word.to_string(), (id as i32, cost));
            true
        });
        map
    }

    pub fn load(fname: &str) -> Result<WordcntUnigram> {
        info!("Reading {}", fname);
        let mut marisa = Marisa::default();
        marisa.load(fname)?;

        let map = Self::_to_count_hashmap(&marisa);

        // 総出現単語数
        let c = map.iter().map(|(_, (_, cnt))| *cnt).sum();
        // 単語の種類数
        let v = map.keys().count();

        let default_cost = calc_cost(0, c, v as u32);
        let default_cost_for_short = calc_cost(1, c, v as u32);

        Ok(WordcntUnigram {
            marisa,
            default_cost,
            default_cost_for_short,
            c,
            v: v as u32,
        })
    }
}

impl SystemUnigramLM for WordcntUnigram {
    fn get_default_cost(&self) -> f32 {
        self.default_cost
    }

    fn get_default_cost_for_short(&self) -> f32 {
        self.default_cost_for_short
    }

    /// @return (word_id, score)。
    fn find(&self, word: &str) -> Option<(i32, f32)> {
        let marisa = &self.marisa;
        assert_ne!(word.len(), 0);

        let key = [word.as_bytes(), b"\xff"].concat();
        let mut word_id: usize = usize::MAX;
        let mut score = u32::MAX;
        marisa.predictive_search(key.as_slice(), |word, id| {
            word_id = id;

            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            score = u32::from_le_bytes(bytes);
            false
        });
        if word_id != usize::MAX {
            Some((word_id as i32, calc_cost(score, self.c, self.v)))
        } else {
            None
        }
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        let mut map = HashMap::new();
        self.marisa.predictive_search("".as_bytes(), |word, id| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            let word = String::from_utf8_lossy(&word[0..idx]);
            let cnt = u32::from_le_bytes(bytes);
            map.insert(
                word.to_string(),
                (id as i32, calc_cost(cnt, self.c, self.v)),
            );
            true
        });
        map
    }
}
