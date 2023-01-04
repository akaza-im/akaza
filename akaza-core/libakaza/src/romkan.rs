use std::collections::HashMap;

use regex::{Captures, Regex};

fn default_romkan_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("xa", "ぁ"),
        ("a", "あ"),
        ("xi", "ぃ"),
        ("i", "い"),
        ("xu", "ぅ"),
        ("u", "う"),
        ("vu", "う゛"),
        ("va", "う゛ぁ"),
        ("vi", "う゛ぃ"),
        ("ve", "う゛ぇ"),
        ("vo", "う゛ぉ"),
        ("xe", "ぇ"),
        ("e", "え"),
        ("xo", "ぉ"),
        ("o", "お"),
        ("ka", "か"),
        ("ga", "が"),
        ("ki", "き"),
        ("kya", "きゃ"),
        ("kyu", "きゅ"),
        ("kyo", "きょ"),
        ("gi", "ぎ"),
        ("gya", "ぎゃ"),
        ("gyu", "ぎゅ"),
        ("gyo", "ぎょ"),
        ("ku", "く"),
        ("gu", "ぐ"),
        ("ke", "け"),
        ("ge", "げ"),
        ("ko", "こ"),
        ("go", "ご"),
        ("sa", "さ"),
        ("za", "ざ"),
        ("shi", "し"),
        ("sha", "しゃ"),
        ("shu", "しゅ"),
        ("si", "し"),
        ("sya", "しゃ"),
        ("syu", "しゅ"),
        ("sho", "しょ"),
        ("ji", "じ"),
        ("ja", "じゃ"),
        ("ju", "じゅ"),
        ("jo", "じょ"),
        ("syo", "しょ"),
        ("zi", "じ"),
        ("zya", "じゃ"),
        ("zyu", "じゅ"),
        ("zyo", "じょ"),
        ("su", "す"),
        ("zu", "ず"),
        ("se", "せ"),
        ("ze", "ぜ"),
        ("so", "そ"),
        ("zo", "ぞ"),
        ("ta", "た"),
        ("da", "だ"),
        ("chi", "ち"),
        ("cha", "ちゃ"),
        ("chu", "ちゅ"),
        ("ti", "ち"),
        ("tya", "ちゃ"),
        ("tyu", "ちゅ"),
        ("cho", "ちょ"),
        ("di", "ぢ"),
        ("dya", "ぢゃ"),
        ("dyu", "ぢゅ"),
        ("dyo", "ぢょ"),
        ("tyo", "ちょ"),
        ("xtsu", "っ"),
        ("xtu", "っ"),
        ("vvu", "っう゛"),
        ("vva", "っう゛ぁ"),
        ("vvi", "っう゛ぃ"),
        ("vve", "っう゛ぇ"),
        ("vvo", "っう゛ぉ"),
        ("kka", "っか"),
        ("gga", "っが"),
        ("kki", "っき"),
        ("kkya", "っきゃ"),
        ("kkyu", "っきゅ"),
        ("kkyo", "っきょ"),
        ("ggi", "っぎ"),
        ("ggya", "っぎゃ"),
        ("ggyu", "っぎゅ"),
        ("ggyo", "っぎょ"),
        ("kku", "っく"),
        ("ggu", "っぐ"),
        ("kke", "っけ"),
        ("gge", "っげ"),
        ("kko", "っこ"),
        ("ggo", "っご"),
        ("ssa", "っさ"),
        ("zza", "っざ"),
        ("sshi", "っし"),
        ("ssha", "っしゃ"),
        ("ssi", "っし"),
        ("ssya", "っしゃ"),
        ("sshu", "っしゅ"),
        ("ssho", "っしょ"),
        ("ssyu", "っしゅ"),
        ("ssyo", "っしょ"),
        ("jji", "っじ"),
        ("jja", "っじゃ"),
        ("jju", "っじゅ"),
        ("jjo", "っじょ"),
        ("zzi", "っじ"),
        ("zzya", "っじゃ"),
        ("zzyu", "っじゅ"),
        ("zzyo", "っじょ"),
        ("ssu", "っす"),
        ("zzu", "っず"),
        ("sse", "っせ"),
        ("zze", "っぜ"),
        ("sso", "っそ"),
        ("zzo", "っぞ"),
        ("tta", "った"),
        ("dda", "っだ"),
        ("cchi", "っち"),
        ("tti", "っち"),
        ("ccha", "っちゃ"),
        ("cchu", "っちゅ"),
        ("ccho", "っちょ"),
        ("ddi", "っぢ"),
        ("ttya", "っちゃ"),
        ("ttyu", "っちゅ"),
        ("ttyo", "っちょ"),
        ("ddya", "っぢゃ"),
        ("ddyu", "っぢゅ"),
        ("ddyo", "っぢょ"),
        ("ttsu", "っつ"),
        ("ttu", "っつ"),
        ("ddu", "っづ"),
        ("tte", "って"),
        ("dde", "っで"),
        ("tto", "っと"),
        ("ddo", "っど"),
        ("hha", "っは"),
        ("bba", "っば"),
        ("ppa", "っぱ"),
        ("hhi", "っひ"),
        ("hhya", "っひゃ"),
        ("hhyu", "っひゅ"),
        ("hhyo", "っひょ"),
        ("bbi", "っび"),
        ("bbya", "っびゃ"),
        ("bbyu", "っびゅ"),
        ("bbyo", "っびょ"),
        ("ppi", "っぴ"),
        ("ppya", "っぴゃ"),
        ("ppyu", "っぴゅ"),
        ("ppyo", "っぴょ"),
        ("ffu", "っふ"),
        ("hhu", "っふ"),
        ("ffa", "っふぁ"),
        ("ffi", "っふぃ"),
        ("ffe", "っふぇ"),
        ("ffo", "っふぉ"),
        ("bbu", "っぶ"),
        ("ppu", "っぷ"),
        ("hhe", "っへ"),
        ("bbe", "っべ"),
        ("ppe", "っぺ"),
        ("hho", "っほ"),
        ("bbo", "っぼ"),
        ("ppo", "っぽ"),
        ("yya", "っや"),
        ("yyu", "っゆ"),
        ("yyo", "っよ"),
        ("rra", "っら"),
        ("rri", "っり"),
        ("rrya", "っりゃ"),
        ("rryu", "っりゅ"),
        ("rryo", "っりょ"),
        ("rru", "っる"),
        ("rre", "っれ"),
        ("rro", "っろ"),
        ("tu", "つ"),
        ("tsu", "つ"),
        ("du", "づ"),
        ("te", "て"),
        ("de", "で"),
        ("to", "と"),
        ("do", "ど"),
        ("na", "な"),
        ("ni", "に"),
        ("nya", "にゃ"),
        ("nyu", "にゅ"),
        ("nyo", "にょ"),
        ("nu", "ぬ"),
        ("ne", "ね"),
        ("no", "の"),
        ("ha", "は"),
        ("ba", "ば"),
        ("pa", "ぱ"),
        ("hi", "ひ"),
        ("hya", "ひゃ"),
        ("hyu", "ひゅ"),
        ("hyo", "ひょ"),
        ("bi", "び"),
        ("bya", "びゃ"),
        ("byu", "びゅ"),
        ("byo", "びょ"),
        ("pi", "ぴ"),
        ("pya", "ぴゃ"),
        ("pyu", "ぴゅ"),
        ("pyo", "ぴょ"),
        ("fu", "ふ"),
        ("fa", "ふぁ"),
        ("fi", "ふぃ"),
        ("fe", "ふぇ"),
        ("fo", "ふぉ"),
        ("hu", "ふ"),
        ("bu", "ぶ"),
        ("pu", "ぷ"),
        ("he", "へ"),
        ("be", "べ"),
        ("pe", "ぺ"),
        ("ho", "ほ"),
        ("bo", "ぼ"),
        ("po", "ぽ"),
        ("ma", "ま"),
        ("mi", "み"),
        ("mya", "みゃ"),
        ("myu", "みゅ"),
        ("myo", "みょ"),
        ("mu", "む"),
        ("me", "め"),
        ("mo", "も"),
        ("xya", "ゃ"),
        ("ya", "や"),
        ("xyu", "ゅ"),
        ("yu", "ゆ"),
        ("xyo", "ょ"),
        ("yo", "よ"),
        ("ra", "ら"),
        ("ri", "り"),
        ("rya", "りゃ"),
        ("ryu", "りゅ"),
        ("ryo", "りょ"),
        ("ru", "る"),
        ("re", "れ"),
        ("ro", "ろ"),
        ("xwa", "ゎ"),
        ("wa", "わ"),
        ("wo", "を"),
        ("n", "ん"),
        ("n'", "ん"),
        ("nn", "ん"),
        ("dyi", "でぃ"),
        ("-", "ー"),
        ("che", "ちぇ"),
        ("tye", "ちぇ"),
        ("cche", "っちぇ"),
        ("ttye", "っちぇ"),
        ("je", "じぇ"),
        ("zye", "じぇ"),
        ("zzye", "っじぇ"),
        ("dha", "でゃ"),
        ("dhi", "でぃ"),
        ("dhu", "でゅ"),
        ("dhe", "でぇ"),
        ("dho", "でょ"),
        ("ddha", "っでゃ"),
        ("ddhi", "っでぃ"),
        ("ddhu", "っでゅ"),
        ("ddhe", "っでぇ"),
        ("ddho", "っでょ"),
        ("tha", "てゃ"),
        ("thi", "てぃ"),
        ("thu", "てゅ"),
        ("the", "てぇ"),
        ("tho", "てょ"),
        ("ttha", "ってゃ"),
        ("tthi", "ってぃ"),
        ("tthu", "ってゅ"),
        ("tthe", "ってぇ"),
        ("ttho", "ってょ"),
        (".", "。"),
        (",", "、"),
        ("[", "「"),
        ("]", "」"),
        ("z[", "『"),
        ("z-", "〜"),
        ("z.", "…"),
        ("z,", "‥"),
        ("zh", "←"),
        ("zj", "↓"),
        ("zk", "↑"),
        ("zl", "→"),
        ("z]", "』"),
        ("z/", "・"),
        ("wi", "うぃ"),
        ("we", "うぇ"),
    ])
}

pub struct RomKanConverter {
    romkan_pattern: Regex,
    romkan_map: HashMap<&'static str, &'static str>,
}

impl Default for RomKanConverter {
    fn default() -> RomKanConverter {
        let romkan_map = default_romkan_map().clone();
        let mut romas = Vec::from_iter(romkan_map.keys());
        romas.sort_by_key(|a| std::cmp::Reverse(a.len()));
        let mut pattern = String::from("(");
        for x in romas {
            pattern += &regex::escape(x);
            pattern += "|";
        }
        pattern += ".)";

        let romkan_pattern = Regex::new(&pattern).unwrap();
        RomKanConverter {
            romkan_pattern,
            romkan_map,
        }
    }
}

impl RomKanConverter {
    pub fn new() -> RomKanConverter {
        Self::default()
    }

    pub fn to_hiragana(&self, src: &str) -> String {
        let src = src.to_ascii_lowercase();
        let src = src.replace("nn", "n");
        let retval = self.romkan_pattern.replace_all(&src, |caps: &Captures| {
            let rom = caps.get(1).unwrap().as_str();
            if let Some(e) = self.romkan_map.get(rom) {
                e.to_string()
            } else {
                rom.to_string()
            }
        });
        retval.into_owned()
    }

    // TODO https://github.com/tokuhirom/akaza/blob/kanakanji/libakaza/src/romkan.cc#L79-L95
    // TODO https://github.com/tokuhirom/akaza/blob/kanakanji/libakaza/t/07_romkan.cc
    pub fn remove_last_char(&self, src: &String) -> String {
        return if src.is_empty() || src.char_indices().count() == 1 {
            String::new()
        } else {
            let (i, _) = src.char_indices().last().unwrap();
            let p = &src[0..i];
            String::from(p)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romkan() {
        let converter = RomKanConverter::new();
        assert_eq!(converter.to_hiragana("aiu"), "あいう");
    }

    #[test]
    fn test_all() {
        let data = [
            ("a", "あ"),
            ("ba", "ば"),
            ("hi", "ひ"),
            ("wahaha", "わはは"),
            ("thi", "てぃ"),
            ("better", "べってr"),
            ("[", "「"),
            ("]", "」"),
            ("wo", "を"),
            ("du", "づ"),
            ("we", "うぇ"),
            ("di", "ぢ"),
            ("fu", "ふ"),
            ("ti", "ち"),
            ("wi", "うぃ"),
            ("we", "うぇ"),
            ("wo", "を"),
            ("z,", "‥"),
            ("z.", "…"),
            ("z/", "・"),
            ("z[", "『"),
            ("z]", "』"),
            ("du", "づ"),
            ("di", "ぢ"),
            ("fu", "ふ"),
            ("ti", "ち"),
            ("wi", "うぃ"),
            ("we", "うぇ"),
            ("wo", "を"),
            ("sorenawww", "それなwww"),
            ("komitthi", "こみってぃ"),
            ("ddha", "っでゃ"),
            ("zzye", "っじぇ"),
        ];
        let converter = RomKanConverter::new();
        for (rom, kana) in data {
            assert_eq!(converter.to_hiragana(rom), kana);
        }
    }
}
