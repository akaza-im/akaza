/// hogen と入力された場合、"ほげn" と表示する。
/// hogena となったら "ほげな"
/// hogenn となったら "ほげん" と表示する必要があるため。
/// 「ん」と一旦表示された後に「な」に変化したりすると気持ち悪く感じる。
/// "meny" のときは "めny" と表示すべき。
use regex::Regex;

pub struct ConsonantSuffixExtractor {
    pattern: Regex,
}

impl Default for ConsonantSuffixExtractor {
    fn default() -> ConsonantSuffixExtractor {
        let pattern = Regex::new("^(.*?)([qwrtypsdfghjklzxcvbmn]+)$").unwrap();
        ConsonantSuffixExtractor { pattern }
    }
}

impl ConsonantSuffixExtractor {
    /// "めny" を ("め", "ny") に分解する。
    // (preedit, suffix) の形で返す。
    pub fn extract(&self, src: &str) -> (String, String) {
        if src.ends_with("nn") {
            (src.to_string(), "".to_string())
        } else if let Some(p) = self.pattern.captures(src) {
            (
                p.get(1).unwrap().as_str().to_string(),
                p.get(2).unwrap().as_str().to_string(),
            )
        } else {
            (src.to_string(), "".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consonant() {
        let cse = ConsonantSuffixExtractor::default();
        assert_eq!(cse.extract("meny"), ("me".to_string(), "ny".to_string()));
        assert_eq!(cse.extract("menn"), ("menn".to_string(), "".to_string()));
        assert_eq!(cse.extract("memo"), ("memo".to_string(), "".to_string()));
    }
}
