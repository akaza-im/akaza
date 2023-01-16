#[allow(unused_imports)]
use chrono::{DateTime, Local, TimeZone};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub surface: String,
    pub yomi: String,
    pub cost: f32,
}

impl Candidate {
    pub(crate) fn key(&self) -> String {
        self.surface.to_string() + "/" + self.yomi.as_str()
    }
}

impl Candidate {
    pub fn new(yomi: &str, surface: &str, cost: f32) -> Candidate {
        Candidate {
            yomi: yomi.to_string(),
            surface: surface.to_string(),
            cost,
        }
    }

    /// 動的なエントリーも考慮した上での surface を得る。
    pub fn surface_with_dynamic(&self) -> String {
        if self.surface.starts_with("(*(*(") {
            match self.surface.as_str() {
                "(*(*(TODAY-HYPHEN" => now().format("%Y-%m-%d").to_string(),
                "(*(*(TODAY-SLASH" => now().format("%Y/%m/%d").to_string(),
                "(*(*(TODAY-KANJI" => now().format("%Y年%m月%d日").to_string(),
                "(*(*(NOW-KANJI" => now().format("%H時%M分").to_string(),
                _ => "不明な動的変換: ".to_string() + self.surface.as_str(),
            }
        } else {
            self.surface.to_string()
        }
    }
}

#[cfg(not(test))]
fn now() -> DateTime<Local> {
    Local::now()
}

#[cfg(test)]
fn now() -> DateTime<Local> {
    Local.with_ymd_and_hms(2023, 1, 16, 15, 14, 16).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::graph::candidate::Candidate;

    #[test]
    fn dynamic() {
        fn test(surface: &str) -> String {
            Candidate::new("きょう", surface, 0.0_f32).surface_with_dynamic()
        }

        assert_eq!(test("(*(*(TODAY-HYPHEN"), "2023-01-16");
        assert_eq!(test("(*(*(TODAY-SLASH"), "2023/01/16");
        assert_eq!(test("(*(*(TODAY-KANJI"), "2023年01月16日");
        assert_eq!(test("(*(*(NOW-KANJI"), "15時14分");
    }
}
