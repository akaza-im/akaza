use std::fs::File;
use std::time::SystemTime;

use anyhow::Context;
use kelp::{kata2hira, ConvOption};
use vibrato::{Dictionary, Tokenizer};

use crate::tokenizer::base::{merge_terms_ipadic, AkazaTokenizer, IntermediateToken};

pub struct VibratoTokenizer {
    tokenizer: Tokenizer,
}

impl VibratoTokenizer {
    pub fn new() -> anyhow::Result<VibratoTokenizer> {
        // システム辞書のロードには14秒ぐらいかかります。
        let dictpath = "work/mecab/ipadic-mecab-2_7_0/system.dic";
        let t1 = SystemTime::now();
        let dict = Dictionary::read(File::open(dictpath)?)?;
        let t2 = SystemTime::now();
        println!(
            "Loaded {} in {}msec",
            dictpath,
            t2.duration_since(t1)?.as_millis()
        );

        let dict = dict
            .reset_user_lexicon_from_reader(Some(File::open(
                "jawiki-kana-kanji-dict/mecab-userdic.csv",
            )?))
            .with_context(|| "Opening userdic")?;

        let tokenizer = vibrato::Tokenizer::new(dict);

        Ok(VibratoTokenizer { tokenizer })
    }
}

impl AkazaTokenizer for VibratoTokenizer {
    /// Vibrato を利用してファイルをアノテーションします。
    fn tokenize(&self, src: &str) -> anyhow::Result<String> {
        let mut worker = self.tokenizer.new_worker();

        worker.reset_sentence(src);
        worker.tokenize();

        let mut intermediates: Vec<IntermediateToken> = Vec::new();

        // Vibrato/mecab の場合、接尾辞などが細かく分かれることは少ないが、
        // 一方で、助詞/助動詞などが細かくわかれがち。
        for i in 0..worker.num_tokens() {
            let token = worker.token(i);
            let feature: Vec<&str> = token.feature().split(',').collect();
            // if feature.len() <= 7 {
            //     println!("Too few features: {}/{}", token.surface(), token.feature())
            // }

            let hinshi = feature[0];
            let subhinshi = if feature.len() > 2 { feature[1] } else { "UNK" };
            let yomi = if feature.len() > 7 {
                feature[7]
            } else {
                // 読みがな不明なもの。固有名詞など。
                // サンデフィヨルド・フォトバル/名詞,固有名詞,組織,*,*,*,*
                token.surface()
            };
            let yomi = kata2hira(yomi, ConvOption::default());
            let intermediate = IntermediateToken::new(
                token.surface().to_string(),
                yomi.to_string(),
                hinshi.to_string(),
                subhinshi.to_string(),
            );
            intermediates.push(intermediate);
            // println!("{}/{}/{}", token.surface(), hinshi, yomi);
        }

        Ok(merge_terms_ipadic(intermediates))
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let runner = VibratoTokenizer::new()?;
        runner.tokenize("私の名前は中野です。")?;
        Ok(())
    }

    #[test]
    fn test_merge() -> anyhow::Result<()> {
        /*
           書いていたものである
           書い    動詞,自立,*,*,五段・カ行イ音便,連用タ接続,書く,カイ,カイ
           て      助詞,接続助詞,*,*,*,*,て,テ,テ
           い      動詞,非自立,*,*,一段,連用形,いる,イ,イ
           た      助動詞,*,*,*,特殊・タ,基本形,た,タ,タ
           もの    名詞,非自立,一般,*,*,*,もの,モノ,モノ
           で      助動詞,*,*,*,特殊・ダ,連用形,だ,デ,デ
           ある    助動詞,*,*,*,五段・ラ行アル,基本形,ある,アル,アル
           EOS
        */
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();

        let runner = VibratoTokenizer::new()?;
        assert_eq!(
            runner.tokenize("書いていたものである")?,
            "書いて/かいて いた/いた もの/もの である/である"
        );
        Ok(())
    }
}
