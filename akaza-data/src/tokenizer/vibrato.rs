use std::fs::File;
use std::time::SystemTime;

use anyhow::Context;
use kelp::{kata2hira, ConvOption};
use log::info;
use vibrato::{Dictionary, Tokenizer};

use crate::tokenizer::base::{merge_terms_ipadic, AkazaTokenizer, IntermediateToken};

pub struct VibratoTokenizer {
    tokenizer: Tokenizer,
}

impl VibratoTokenizer {
    pub fn new(dictpath: &str, user_dict: Option<String>) -> anyhow::Result<VibratoTokenizer> {
        // システム辞書のロードには14秒ぐらいかかります。
        let t1 = SystemTime::now();
        let mut dict = Dictionary::read(File::open(dictpath)?)?;
        let t2 = SystemTime::now();
        println!(
            "Loaded {} in {}msec",
            dictpath,
            t2.duration_since(t1)?.as_millis()
        );

        // ユーザー辞書として jawiki-kana-kanji-dict を使うと
        // 変な単語を間違って覚えることがあるので、
        // トーカナイズフェーズには入れないこと。
        if let Some(user_dict) = user_dict {
            info!("Loading user dictionary: {}", user_dict);
            dict = dict
                .reset_user_lexicon_from_reader(Some(File::open(user_dict)?))
                .with_context(|| "Opening userdic")?;
        }

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
            let subsubhinshi = if feature.len() > 3 { feature[2] } else { "UNK" };
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
                subsubhinshi.to_string(),
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
        let runner = VibratoTokenizer::new("work/vibrato/ipadic-mecab-2_7_0/system.dic", None)?;
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

        let runner = VibratoTokenizer::new("work/vibrato/ipadic-mecab-2_7_0/system.dic", None)?;
        assert_eq!(
            runner.tokenize("書いていたものである")?,
            "書いて/かいて いた/いた もの/もの である/である"
        );
        Ok(())
    }

    #[test]
    fn test_iika() -> anyhow::Result<()> {
        // 井伊家が ipadic だと いい/か になるが、「か」が接尾なので、
        // 複合語化されてしまう。これはひじょうに問題である。
        // 「いいか」というのはよく出て来る表現なので。。
        // しかも「井伊家」は wikipedia でもよく出ているので、割とコストが低くなってしまう。
        // 井伊家だけに限った問題ではないので、mecab への辞書登録ではカバーが難しい。
        // よって、接尾の「家」は特別扱いして、固有名詞,人名の場合のあとにくる「家」は「け」と読むようにする。

        /*
        井伊家
        井伊    名詞,固有名詞,人名,姓,*,*,井伊,イイ,イイ
        家      名詞,接尾,一般,*,*,*,家,カ,カ
        EOS
        */

        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();

        let runner = VibratoTokenizer::new("work/vibrato/ipadic-mecab-2_7_0/system.dic", None)?;
        assert_eq!(runner.tokenize("井伊家")?, "井伊家/いいけ");
        Ok(())
    }
}
