use crate::UNKNOWN_WORD_ID;

/**
 * unigram 言語モデル。
 * 「漢字」に対して、発生確率スコアを保持している。
 */
pub struct SystemUnigramLMBuilder {
    data: Vec<(String, f32)>,
}

impl SystemUnigramLMBuilder {
    pub fn new() -> SystemUnigramLMBuilder {
        SystemUnigramLMBuilder { data: Vec::new() }
    }

    pub fn add(&mut self, word: &String, score: f32) {
        self.data.push((word.clone(), score));
    }

    pub fn save(&self, fname: &String) -> Result<(), String> {
        return match sled::open(fname) {
            Ok(db) => {
                let mut word_id: i32 = 1;
                for p in &self.data {
                    let (word, score) = p;
                    let result = db.insert(
                        word,
                        [word_id.to_le_bytes(), score.to_le_bytes()]
                            .concat()
                            .to_vec(),
                    );
                    if let Err(result) = result {
                        return Err(result.to_string());
                    }
                    word_id += 1;
                }
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        };
    }
}

pub struct SystemUnigramLM {
    pub db: sled::Db,
}

impl SystemUnigramLM {
    pub(crate) fn get_default_cost(&self) -> f32 {
        todo!()
    }
    pub(crate) fn get_default_cost_for_short(&self) -> f32 {
        todo!()
    }
}

impl SystemUnigramLM {
    pub fn num_keys(&self) -> usize {
        return self.db.len();
    }

    pub fn load(fname: &String) -> Result<SystemUnigramLM, String> {
        println!("Reading {}", fname);
        return match sled::open(fname) {
            Ok(db) => Ok(SystemUnigramLM { db }),
            Err(err) => Err(err.to_string()),
        };
    }

    /// @return (word_id, score)。
    pub fn find_unigram(&self, word: &String) -> Result<(i32, f32), String> {
        return match self.db.get(word) {
            Ok(Some(f)) => {
                let word_id = i32::from_le_bytes(f[0..4].try_into().unwrap());
                let score = f32::from_le_bytes(f[4..8].try_into().unwrap());
                Ok((word_id, score))
            }
            Ok(None) => Ok((UNKNOWN_WORD_ID, 0_f32)),
            Err(err) => Err(err.to_string()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut builder = SystemUnigramLMBuilder::new();
        builder.add(&"hello".to_string(), 0.4);
        builder.add(&"world".to_string(), 0.2);
        let got = builder.save(&"/tmp/system_unigram_lm.sled".to_string());
        assert_eq!(got.is_ok(), true);

        let lm = SystemUnigramLM::load(&"/tmp/system_unigram_lm.sled".to_string()).unwrap();
        {
            let (word_id, score) = lm.find_unigram(&"unknown".to_string()).unwrap();
            assert_eq!(word_id, UNKNOWN_WORD_ID);
            assert_eq!(score, 0_f32);
        }
        {
            let (word_id, score) = lm.find_unigram(&"hello".to_string()).unwrap();
            assert_eq!(word_id, 1);
            assert_eq!(score, 0.4_f32);
        }
    }
}
