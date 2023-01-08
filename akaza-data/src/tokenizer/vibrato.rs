use anyhow::Context;
use kelp::{kata2hira, ConvOption};
use log::info;
use std::fs::File;
use vibrato::{Dictionary, Tokenizer};

pub struct VibratoTokenizer {
    tokenizer: Tokenizer,
}

impl VibratoTokenizer {
    pub fn new() -> anyhow::Result<VibratoTokenizer> {
        let dictpath = "work/mecab/ipadic-mecab-2_7_0/system.dic";
        let dict = Dictionary::read(File::open(dictpath)?)?;
        info!("Loaded {}", dictpath);
        let dict = dict
            .reset_user_lexicon_from_reader(Some(File::open(
                "jawiki-kana-kanji-dict/mecab-userdic.csv",
            )?))
            .with_context(|| "Opening userdic")?;

        let tokenizer = vibrato::Tokenizer::new(dict);

        Ok(VibratoTokenizer { tokenizer })
    }

    /// Vibrato を利用してファイルをアノテーションします。
    pub fn tokenize(&self, src: &str) -> anyhow::Result<String> {
        let mut worker = self.tokenizer.new_worker();

        worker.reset_sentence(src);
        worker.tokenize();

        let mut buf = String::new();

        // TODO 連結処理的なことが必要ならする。
        // Vibrato/mecab の場合、接尾辞などが細かく分かれることは少ないが、
        // 一方で、助詞/助動詞などが細かくわかれがち。
        for i in 0..worker.num_tokens() {
            let token = worker.token(i);
            let feature: Vec<&str> = token.feature().split(',').collect();
            // if feature.len() <= 7 {
            //     println!("Too few features: {}/{}", token.surface(), token.feature())
            // }

            // let hinshi = feature[0];
            let yomi = if feature.len() > 7 {
                feature[7]
            } else {
                // 読みがな不明なもの。固有名詞など。
                // サンデフィヨルド・フォトバル/名詞,固有名詞,組織,*,*,*,*
                token.surface()
            };
            let yomi = kata2hira(yomi, ConvOption::default());
            buf += format!("{}/{} ", token.surface(), yomi).as_str();
            // println!("{}/{}/{}", token.surface(), hinshi, yomi);
        }

        Ok(buf.trim().to_string() + "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let runner = VibratoTokenizer::new()?;
        runner.tokenize("私の名前は中野です。")?;
        Ok(())
    }
}
