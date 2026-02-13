use std::borrow::Cow;
use std::fs::File;
use std::time::SystemTime;

use anyhow::Context;
use log::{debug, info};
use vibrato::{Dictionary, Tokenizer};

use crate::tokenizer::base::{
    kata2hira_into, merge_terms_ipadic, AkazaTokenizer, IntermediateToken,
};

pub struct VibratoTokenizer {
    tokenizer: Tokenizer,
}

impl VibratoTokenizer {
    pub fn new(dictpath: &str, user_dict: Option<String>) -> anyhow::Result<VibratoTokenizer> {
        // システム辞書のロードには14秒ぐらいかかります。
        let t1 = SystemTime::now();
        let mut dict = Dictionary::read(File::open(dictpath)?)?;
        let t2 = SystemTime::now();
        debug!(
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
    fn tokenize(&self, src: &str, kana_preferred: bool) -> anyhow::Result<String> {
        let mut worker = self.tokenizer.new_worker();

        worker.reset_sentence(src);
        worker.tokenize();

        let mut intermediates: Vec<IntermediateToken> = Vec::with_capacity(worker.num_tokens());
        let mut yomi_buf = String::new();

        // Vibrato/mecab の場合、接尾辞などが細かく分かれることは少ないが、
        // 一方で、助詞/助動詞などが細かくわかれがち。
        for i in 0..worker.num_tokens() {
            let token = worker.token(i);
            let feature = token.feature();
            let mut parts = feature.split(',');

            let hinshi = parts.next().unwrap_or("UNK");
            let subhinshi = parts.next().unwrap_or("UNK");
            let subsubhinshi = parts.next().unwrap_or("UNK");
            // feature[3]..feature[6] をスキップ
            let yomi_raw = parts.nth(4).unwrap_or(token.surface());
            kata2hira_into(yomi_raw, &mut yomi_buf);
            let surface = if should_be_kana(kana_preferred, hinshi, subhinshi) {
                Cow::Owned(yomi_buf.clone())
            } else {
                Cow::Owned(token.surface().to_string())
            };
            let yomi = std::mem::take(&mut yomi_buf);
            let intermediate = IntermediateToken {
                surface,
                yomi: Cow::Owned(yomi),
                hinshi,
                subhinshi,
                subsubhinshi,
            };
            intermediates.push(intermediate);
        }

        Ok(merge_terms_ipadic(&intermediates))
    }
}

/// かな優先モードの処理
fn should_be_kana(kana_preferred: bool, hinshi: &str, subhinshi: &str) -> bool {
    if !kana_preferred {
        return false;
    }

    // 貴方    名詞,代名詞,一般,*,*,*,貴方,アナタ,アナタ
    subhinshi == "代名詞"
        // 美しい  形容詞,自立,*,*,形容詞・イ段,基本形,美しい,ウツクシイ,ウツ クシイ
        || hinshi == "形容詞"
        // 到底    副詞,一般,*,*,*,*,到底,トウテイ,トーテイ
        || hinshi == "副詞"
        // 及び    接続詞,*,*,*,*,*,及び,オヨビ,オヨビ
        || hinshi == "接続詞"
        // 嗚呼    感動詞,*,*,*,*,*,嗚呼,アア,アー
        || hinshi == "感動詞"
        // 仰ぐ    動詞,自立,*,*,五段・ガ行,基本形,仰ぐ,アオグ,アオグ
        || hinshi == "動詞"
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    #[test]
    fn test_should_be_kana() -> anyhow::Result<()> {
        assert!(!should_be_kana(false, "形容詞", "自立"));
        assert!(should_be_kana(true, "形容詞", "自立"));
        Ok(())
    }

    #[test]
    fn test_with_kana() -> anyhow::Result<()> {
        let dict_path = "work/vibrato/ipadic-mecab-2_7_0/system.dic";
        let abs_path = std::env::current_dir()?.join(dict_path);
        eprintln!("Attempting to load dictionary from: {:?}", abs_path);
        let runner = VibratoTokenizer::new(abs_path.to_str().unwrap(), None)
            .with_context(|| format!("Failed to load dictionary from: {:?}", abs_path))?;
        let got = runner.tokenize("私の名前は中野です。", true)?;
        assert_eq!(
            got,
            "わたし/わたし の/の 名前/なまえ は/は 中野/なかの です/です 。/。"
        );
        Ok(())
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let dict_path = "work/vibrato/ipadic-mecab-2_7_0/system.dic";
        let abs_path = std::env::current_dir()?.join(dict_path);
        eprintln!("Attempting to load dictionary from: {:?}", abs_path);
        let runner = VibratoTokenizer::new(abs_path.to_str().unwrap(), None)
            .with_context(|| format!("Failed to load dictionary from: {:?}", abs_path))?;
        runner.tokenize("私の名前は中野です。", false)?;
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

        let dict_path = "work/vibrato/ipadic-mecab-2_7_0/system.dic";
        let abs_path = std::env::current_dir()?.join(dict_path);
        eprintln!("Attempting to load dictionary from: {:?}", abs_path);
        let runner = VibratoTokenizer::new(abs_path.to_str().unwrap(), None)
            .with_context(|| format!("Failed to load dictionary from: {:?}", abs_path))?;
        assert_eq!(
            runner.tokenize("書いていたものである", false)?,
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

        let dict_path = "work/vibrato/ipadic-mecab-2_7_0/system.dic";
        let abs_path = std::env::current_dir()?.join(dict_path);
        eprintln!("Attempting to load dictionary from: {:?}", abs_path);
        let runner = VibratoTokenizer::new(abs_path.to_str().unwrap(), None)
            .with_context(|| format!("Failed to load dictionary from: {:?}", abs_path))?;
        assert_eq!(runner.tokenize("井伊家", false)?, "井伊家/いいけ");
        Ok(())
    }
}
