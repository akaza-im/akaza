use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::input_style::InputStyle::Romaji;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum InputStyle {
    /// ローマ字入力
    Romaji,
    /// かな入力(JIS X 6002)
    Kana,
}

impl Default for InputStyle {
    fn default() -> Self {
        Romaji
    }
}

pub struct InputStyleMapper {
    jisx6002: HashMap<char, &'static str>,
    dakuten_map: HashMap<&'static str, &'static str>,
    handakuten_map: HashMap<&'static str, &'static str>,
}

#[rustfmt::skip]
impl Default for InputStyleMapper {
    fn default() -> Self {
        InputStyleMapper {
            jisx6002: HashMap::from([
                // 1列目
                ('1', "ぬ"),
                ('2', "ふ"),
                ('3', "あ"),
                ('4', "う"),
                ('5', "え"),
                ('6', "お"),
                ('7', "や"),
                ('8', "ゆ"),
                ('9', "よ"),
                ('0', "わ"),
                ('-', "ほ"),
                ('^', "へ"),
                ('\\', "ー"),
                // 2列目
                ('q', "た"),
                ('w', "て"),
                ('e', "い"),
                ('r', "す"),
                ('t', "か"),
                ('y', "ん"),
                ('u', "な"),
                ('i', "に"),
                ('o', "ら"),
                ('p', "せ"),
                ('@', "゛"),
                ('[', "゜"),
                // 3列目
                ('a', "ち"),
                ('s', "と"),
                ('d', "し"),
                ('f', "は"),
                ('g', "き"),
                ('h', "く"),
                ('j', "ま"),
                ('k', "の"),
                ('l', "り"),
                (';', "れ"),
                (':', "け"),
                (']', "む"),
                // 4列目
                ('z', "つ"),
                ('x', "さ"),
                ('c', "そ"),
                ('v', "ひ"),
                ('b', "こ"),
                ('n', "み"),
                ('m', "も"),
                (',', "ね"),
                ('.', "る"),
                ('/', "め"),
                ('\\', "ろ"),
                // シフトつき
                ('#', "ぁ"),
                ('$', "ぅ"),
                ('%', "ぇ"),
                ('&', "ぉ"),
                ('\'', "ゃ"),
                ('(', "ゅ"),
                (')', "ょ"),
                ('Z', "っ"),
                ('<', "ね"),
                ('>', "る"),
                ('?', "め"),
                ('_', "ろ"),
                ('{', "「"),
                ('}', "」"),
            ]),
            dakuten_map: HashMap::from([
                ("う", "ゔ"),
                ("か", "が"),
                ("き", "ぎ"),
                ("く", "ぐ"),
                ("け", "げ"),
                ("こ", "ご"),
                ("さ", "ざ"),
                ("し", "じ"),
                ("す", "ず"),
                ("せ", "ぜ"),
                ("そ", "ぞ"),
                ("た", "だ"),
                ("ち", "ぢ"),
                ("つ", "づ"),
                ("て", "で"),
                ("と", "ど"),
                ("は", "ば"),
                ("ひ", "び"),
                ("ふ", "ぶ"),
                ("へ", "べ"),
                ("ほ", "ぼ"),
            ]),
            handakuten_map: HashMap::from([
                ("は", "ぱ"),
                ("ひ", "ぴ"),
                ("ふ", "ぷ"),
                ("へ", "ぺ"),
                ("ほ", "ぽ"),
            ]),
        }
    }
}

impl InputStyleMapper {
    /// JIS X 6002
    pub fn kana_input_jis_x_6002(&self, preedit: String, ch: char) -> String {
        if let Some(got) = self.jisx6002.get(&ch) {
            // 濁点の連結処理が必要。
            for (symbol, henkan_map) in [
                ("゛", &self.dakuten_map),    // 濁点
                ("゜", &self.handakuten_map), // 半濁点
            ] {
                if *got == symbol && !preedit.is_empty() {
                    let chars = preedit.chars();
                    let lastch = chars.last().unwrap();
                    return if let Some(dakuten) = henkan_map.get(lastch.to_string().as_str()) {
                        let (a, _) = preedit.char_indices().last().unwrap();
                        preedit[0..a].to_string() + dakuten
                    } else {
                        preedit + *got
                    };
                }
            }
            preedit + *got
        } else {
            preedit + ch.to_string().as_str()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testfoobr() {
        let mapper = InputStyleMapper::default();
        assert_eq!(
            mapper.kana_input_jis_x_6002("か".to_string(), '@'),
            "が".to_string()
        );
        assert_eq!(
            mapper.kana_input_jis_x_6002("あか".to_string(), '@'),
            "あが".to_string()
        );
        assert_eq!(
            mapper.kana_input_jis_x_6002("は".to_string(), '@'),
            "ば".to_string()
        );
        assert_eq!(
            mapper.kana_input_jis_x_6002("は".to_string(), '['),
            "ぱ".to_string()
        );
        assert_eq!(
            mapper.kana_input_jis_x_6002("は".to_string(), 'a'),
            "はち".to_string()
        );
    }
}
