use std::borrow::Cow;
use std::fmt::Write;

pub trait AkazaTokenizer {
    fn tokenize(&self, src: &str, kana_preferred: bool) -> anyhow::Result<String>;
}

/// マージ処理に利用する為の中間表現
#[derive(Debug)]
pub(crate) struct IntermediateToken<'a> {
    pub surface: Cow<'a, str>,
    pub yomi: Cow<'a, str>,
    pub hinshi: &'a str,
    pub subhinshi: &'a str,
    pub subsubhinshi: &'a str,
}

/// カタカナをひらがなに変換する（アロケーションなし、バッファ使い回し）
pub(crate) fn kata2hira_into(s: &str, buf: &mut String) {
    buf.clear();
    buf.reserve(s.len());
    for c in s.chars() {
        let c = match c {
            '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            '\u{30FD}'..='\u{30FE}' => char::from_u32(c as u32 - 0x60).unwrap_or(c), // ヽヾ → ゝゞ
            _ => c,
        };
        buf.push(c);
    }
}

/// カタカナをひらがなに変換する（新しい String を返す版）
pub(crate) fn kata2hira_string(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    for c in s.chars() {
        let c = match c {
            '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            '\u{30FD}'..='\u{30FE}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            _ => c,
        };
        buf.push(c);
    }
    buf
}

/// 特定の品詞をマージする
/// ipadic の品詞体系を対象とする。
pub(crate) fn merge_terms_ipadic(intermediates: &[IntermediateToken]) -> String {
    let mut buf = String::new();
    let mut i = 0;
    while i < intermediates.len() {
        let token = &intermediates[i];
        let mut surface: Cow<str> = Cow::Borrowed(&token.surface);
        let mut yomi: Cow<str> = Cow::Borrowed(&token.yomi);
        let mut prev_token = token;

        let mut j = i + 1;
        while j < intermediates.len() {
            /*
               実施/名詞/サ変接続/じっし
               さ/動詞/自立/さ
               れ/動詞/接尾/れ
               た/助動詞/_/た

               のような場合、"実施,された"に連結したい。

                書い/動詞/自立/かい
                て/助詞/接続助詞/て
                い/動詞/非自立/い
                た/助動詞/_/た
                もの/名詞/非自立/もの
                で/助動詞/_/で
                ある/助動詞/_/ある

                を、"書いて、いた、ものである" ぐらいまで連結する。

                助動詞とその前のトークンを単純に接続すると以下の様なケースで困る。

                鈴鹿医療科学技術大学/名詞/固有名詞/すずかいりょうかがくぎじゅつだいがく
                で/助動詞/_/で
                あっ/助動詞/_/あっ
                た/助動詞/_/た
                が/助詞/接続助詞/が
            */
            let token = &intermediates[j];

            if (token.hinshi == "助動詞"
                && (prev_token.hinshi == "動詞" || prev_token.hinshi == "助動詞"))
                || token.subhinshi == "接続助詞"
                || token.subhinshi == "接尾"
            {
                surface.to_mut().push_str(&token.surface);
                let yomi_part = if *token.surface == *"家"
                    && *token.yomi == *"か"
                    && prev_token.subsubhinshi == "人名"
                {
                    // 人名 + 家 のケースに ipadic だと「か」と読んでしまう
                    // 問題があるので、その場合は「家/け」に読み替える。
                    "け"
                } else {
                    &token.yomi
                };
                yomi.to_mut().push_str(yomi_part);

                j += 1;
                prev_token = token;
            } else {
                break;
            }
        }

        write!(buf, "{surface}/{yomi} ").unwrap();

        i = j;
    }
    // 末尾の空白を除去（新規アロケーションなし）
    let trimmed_len = buf.trim_end().len();
    buf.truncate(trimmed_len);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kata2hira_basic() {
        let mut buf = String::new();
        kata2hira_into("カタカナ", &mut buf);
        assert_eq!(buf, "かたかな");
    }

    #[test]
    fn test_kata2hira_mixed() {
        let mut buf = String::new();
        kata2hira_into("アイウエオあいうえお", &mut buf);
        assert_eq!(buf, "あいうえおあいうえお");
    }

    #[test]
    fn test_kata2hira_ascii() {
        let mut buf = String::new();
        kata2hira_into("ABC123", &mut buf);
        assert_eq!(buf, "ABC123");
    }

    #[test]
    fn test_kata2hira_repeat_marks() {
        let mut buf = String::new();
        kata2hira_into("ヽヾ", &mut buf);
        assert_eq!(buf, "ゝゞ");
    }

    #[test]
    fn test_kata2hira_string_version() {
        assert_eq!(kata2hira_string("カタカナ"), "かたかな");
        assert_eq!(kata2hira_string("テスト"), "てすと");
    }

    #[test]
    fn test_kata2hira_buffer_reuse() {
        let mut buf = String::new();
        kata2hira_into("テスト", &mut buf);
        assert_eq!(buf, "てすと");
        kata2hira_into("カナ", &mut buf);
        assert_eq!(buf, "かな"); // 前回の内容はクリアされている
    }
}
