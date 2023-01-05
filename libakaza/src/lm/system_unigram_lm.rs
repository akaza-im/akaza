use anyhow::Result;
use log::info;
use marisa_sys::{Keyset, Marisa};

/**
 * unigram 言語モデル。
 * 「漢字」に対して、発生確率スコアを保持している。
 */
#[derive(Default)]
pub struct SystemUnigramLMBuilder {
    data: Vec<(String, f32)>,
}

impl SystemUnigramLMBuilder {
    pub fn add(&mut self, word: &str, score: f32) {
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

    pub fn build(&self) -> SystemUnigramLM {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset());
        SystemUnigramLM { marisa }
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

    pub fn load(fname: &str) -> Result<SystemUnigramLM> {
        info!("Reading {}", fname);
        let mut marisa = Marisa::default();
        marisa.load(fname)?;
        Ok(SystemUnigramLM { marisa })
    }

    /// @return (word_id, score)。
    pub fn find(&self, word: &str) -> Option<(i32, f32)> {
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
            Some((kanji_id as i32, score))
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

        let mut builder = SystemUnigramLMBuilder::default();
        builder.add("hello", 0.4);
        builder.add("world", 0.2);
        builder.save(&tmpfile).unwrap();

        let lm = SystemUnigramLM::load(&tmpfile).unwrap();
        {
            let (word_id, score) = lm.find("hello").unwrap();
            assert_eq!(word_id, 0);
            assert_eq!(score, 0.4_f32);
        }
        {
            let p = lm.find("unknown");
            assert_eq!(p, None);
        }
    }
}
