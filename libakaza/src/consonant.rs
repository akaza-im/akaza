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
        let pattern = Regex::new("^(.*?(?:nn)*)([qwrtypsdfghjklzxcvbmn]+)$").unwrap();
        ConsonantSuffixExtractor { pattern }
    }
}

impl ConsonantSuffixExtractor {
    /// "めny" を ("め", "ny") に分解する。
    // (preedit, suffix) の形で返す。
    pub fn extract(&self, src: &str) -> (String, String) {
        if
        // nn は「ん」で確定されている。
        src.ends_with("nn")
            // 矢印などはのこった子音じゃない。
            || src.ends_with("zh")
            || src.ends_with("zj")
            || src.ends_with("zk")
            || src.ends_with("zl")
            || src.ends_with("z[")
            || src.ends_with("z]")
            || src.ends_with("z-")
            || src.ends_with("z.")
            || src.ends_with("z,")
            || src.ends_with("z/")
        {
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
        // うーんwwww の場合、 wwww が suffix であるべき
        assert_eq!(
            cse.extract("u-nnwwww"),
            ("u-nn".to_string(), "wwww".to_string())
        );
        assert_eq!(
            cse.extract("u-nnnnwwww"),
            ("u-nnnn".to_string(), "wwww".to_string())
        );
    }
}
