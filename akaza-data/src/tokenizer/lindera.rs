use kelp::{kata2hira, ConvOption};
use lindera::mode::Mode;
use lindera::tokenizer::{DictionaryConfig, Tokenizer, TokenizerConfig};
use lindera::DictionaryKind;

pub struct LinderaTokenizer {
    tokenizer: Tokenizer,
}

impl LinderaTokenizer {
    fn new() -> anyhow::Result<LinderaTokenizer> {
        let dictionary = DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        };

        let config = TokenizerConfig {
            dictionary,
            user_dictionary: None,
            mode: Mode::Normal,
            with_details: true,
        };

        // create tokenizer
        let tokenizer = Tokenizer::from_config(config)?;
        Ok(LinderaTokenizer { tokenizer })
    }

    fn tokenize(&self, src: &str) -> anyhow::Result<String> {
        // tokenize the text
        let tokens = self.tokenizer.tokenize(src)?;
        let mut buf = String::new();
        for token in tokens {
            let details = token.details.unwrap();
            let surface = token.text;
            let yomi = if details.len() > 7 {
                details[7].to_string()
            } else {
                surface.to_string()
            };
            let yomi = kata2hira(yomi.as_str(), ConvOption::default());
            buf += format!("{}/{} ", surface, yomi).as_str();
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lindera_test() -> anyhow::Result<()> {
        let tokenizer = LinderaTokenizer::new()?;
        let tokens = tokenizer.tokenize(
            "関西国際空港/かんさいこくさいくうこう 限定/げんてい トートバッグ/とーとばっぐ",
        )?;
        assert_eq!(tokens, "");

        Ok(())
    }
}
