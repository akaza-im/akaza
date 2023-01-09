use std::path::PathBuf;

use kelp::{kata2hira, ConvOption};
use lindera::mode::Mode;
use lindera::tokenizer::{DictionaryConfig, Tokenizer, TokenizerConfig, UserDictionaryConfig};
use lindera::DictionaryKind;
use log::{info, trace};

use crate::tokenizer::base::{merge_terms_ipadic, AkazaTokenizer, IntermediateToken};

pub struct LinderaTokenizer {
    tokenizer: Tokenizer,
}

impl LinderaTokenizer {
    pub(crate) fn new(
        dictionary_kind: DictionaryKind,
        user_dictionary_path: Option<PathBuf>,
    ) -> anyhow::Result<LinderaTokenizer> {
        info!("Building tokenizer... with {:?}", user_dictionary_path);

        let dictionary = DictionaryConfig {
            kind: Some(dictionary_kind.clone()),
            path: None,
        };

        let user_dictionary = user_dictionary_path.map(|path| UserDictionaryConfig {
            kind: Some(dictionary_kind),
            path,
        });

        let config = TokenizerConfig {
            dictionary,
            user_dictionary,
            mode: Mode::Normal,
            with_details: true,
        };

        // create tokenizer
        let tokenizer = Tokenizer::from_config(config)?;
        info!("Built tokenizer ");

        Ok(LinderaTokenizer { tokenizer })
    }
}

impl AkazaTokenizer for LinderaTokenizer {
    fn tokenize(&self, src: &str) -> anyhow::Result<String> {
        // tokenize the text
        let tokens = self.tokenizer.tokenize(src)?;

        // 取り扱いやすい中間表現に変更する
        let mut intermediates: Vec<IntermediateToken> = Vec::new();
        for token in tokens {
            let details = token.details.unwrap();
            let surface = token.text;

            let yomi = if details.len() > 7 {
                details[7].to_string()
            } else {
                surface.to_string()
            };
            let yomi = kata2hira(yomi.as_str(), ConvOption::default());

            let hinshi = details[0].to_string();
            let subhinshi = if details.len() > 1 {
                details[1].to_string()
            } else {
                "UNK".to_string()
            };

            trace!("{}/{}/{}/{}", surface, hinshi, subhinshi, yomi);

            intermediates.push(IntermediateToken::new(
                surface.to_string(),
                yomi,
                hinshi,
                subhinshi,
            ));
        }

        Ok(merge_terms_ipadic(intermediates))
    }
}

#[cfg(test)]
mod tests {
    use lindera::DictionaryKind::IPADIC;
    use log::LevelFilter;

    use super::*;

    #[test]
    fn lindera_test() -> anyhow::Result<()> {
        let tokenizer = LinderaTokenizer::new(IPADIC, None)?;
        let tokens = tokenizer.tokenize("関西国際空港限定トートバッグ")?;
        assert_eq!(
            tokens,
            "関西国際空港/かんさいこくさいくうこう 限定/げんてい トートバッグ/とーとばっぐ"
        );

        Ok(())
    }

    #[cfg(test)]
    mod merger {
        use super::*;

        /// かな漢字変換で使うには分割がこまかすぎるので、連結していく。
        #[test]
        fn lindera_merge() -> anyhow::Result<()> {
            /*
               実施/名詞/サ変接続/じっし
               さ/動詞/自立/さ
               れ/動詞/接尾/れ
               た/助動詞/_/た
            */

            let _ = env_logger::builder()
                .filter_level(LevelFilter::Trace)
                .is_test(true)
                .try_init();

            let tokenizer = LinderaTokenizer::new(IPADIC, None)?;
            let tokens = tokenizer.tokenize("実施された")?;
            assert_eq!(tokens, "実施/じっし された/された");

            Ok(())
        }

        /// かな漢字変換で使うには分割がこまかすぎるので、連結していく。
        #[test]
        fn lindera_merge2() -> anyhow::Result<()> {
            let _ = env_logger::builder()
                .filter_level(LevelFilter::Trace)
                .is_test(true)
                .try_init();

            let tokenizer = LinderaTokenizer::new(IPADIC, None)?;
            let tokens = tokenizer.tokenize("小学校")?;
            assert_eq!(tokens, "小学校/しょうがっこう");

            Ok(())
        }

        #[test]
        fn lindera_merge3() -> anyhow::Result<()> {
            let _ = env_logger::builder()
                .filter_level(LevelFilter::Trace)
                .is_test(true)
                .try_init();

            let tokenizer = LinderaTokenizer::new(IPADIC, None)?;
            let tokens = tokenizer.tokenize("書いていたものである")?;
            assert_eq!(tokens, "書いて/かいて いた/いた もの/もの である/である");

            Ok(())
        }

        #[test]
        fn lindera_merge4() -> anyhow::Result<()> {
            let _ = env_logger::builder()
                .filter_level(LevelFilter::Trace)
                .is_test(true)
                .try_init();

            let tokenizer = LinderaTokenizer::new(IPADIC, None)?;
            let tokens = tokenizer.tokenize("鈴鹿医療科学技術大学であったが")?;
            assert_eq!(
                tokens,
                "鈴鹿医療科学技術大学/すずかいりょうかがくぎじゅつだいがく であったが/であったが"
            );

            Ok(())
        }
    }
}
