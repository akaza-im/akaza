use marisa_sys::{Keyset, Marisa};

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
            keyset.push_back(key.as_slice());
        }

        let mut marisa = Marisa::new();
        marisa.build(&keyset);
        marisa.save(fname)?;
        Ok(())
    }
}

pub struct SystemUnigramLM {
    marisa: Marisa,
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
        self.marisa.num_keys()
    }

    pub fn load(fname: &String) -> Result<SystemUnigramLM, String> {
        println!("Reading {}", fname);
        let mut marisa = Marisa::new();
        marisa.load(fname)?;
        Ok(SystemUnigramLM { marisa })
    }

    /// @return (word_id, score)。
    pub fn find(&self, word: &String) -> Option<(usize, f32)> {
        assert_ne!(word.len(), 0);

        let key = [word.as_bytes(), b"\xff"].concat();
        let mut kanji_id: usize = usize::MAX;
        let mut score = f32::MAX;
        self.marisa.predictive_search(key.as_slice(), |word, id| {
            kanji_id = id;

            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            score = f32::from_le_bytes(bytes);
            false
        });
        if kanji_id != usize::MAX {
            Some((kanji_id, score))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test() {
        let named_tmpfile = NamedTempFile::new().unwrap();
        let tmpfile = named_tmpfile.path().to_str().unwrap().to_string();

        let mut builder = SystemUnigramLMBuilder::new();
        builder.add(&"hello".to_string(), 0.4);
        builder.add(&"world".to_string(), 0.2);
        builder.save(&tmpfile).unwrap();

        let lm = SystemUnigramLM::load(&tmpfile).unwrap();
        {
            let (word_id, score) = lm.find(&"hello".to_string()).unwrap();
            assert_eq!(word_id, 0);
            assert_eq!(score, 0.4_f32);
        }
        {
            let p = lm.find(&"unknown".to_string());
            assert_eq!(p, None);
        }
    }
}
