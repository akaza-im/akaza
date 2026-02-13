use std::collections::HashMap;

use anyhow::bail;

pub struct Ari2Nasi {
    boin_map: HashMap<char, &'static str>,
    roman_map: HashMap<&'static str, &'static str>,
}

impl Default for Ari2Nasi {
    fn default() -> Ari2Nasi {
        let boin_map = HashMap::from([
            ('a', "あ"),
            ('i', "い"),
            ('u', "う"),
            ('e', "え"),
            ('o', "お"),
        ]);
        let roman_map = HashMap::from([
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
            ("wi", "うぃ"),
            ("we", "うぇ"),
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
        ]);
        Ari2Nasi {
            boin_map,
            roman_map,
        }
    }
}

impl Ari2Nasi {
    fn expand_okuri(
        &self,
        kana: &str,
        kanjis: &[String],
    ) -> anyhow::Result<Vec<(String, Vec<String>)>> {
        let Some(last_char) = kana.chars().last() else {
            bail!("kana is empty");
        };
        if last_char.is_ascii_alphabetic() {
            if self.boin_map.contains_key(&last_char) {
                // 母音の場合はそのまま平仮名に変換する。
                // e.g. "a" → "あ"
                let okuri = self.boin_map.get(&last_char).unwrap();
                let yomi = &kana[0..kana.len() - last_char.len_utf8()];
                let kanjis = kanjis.iter().map(|f| f.to_string() + *okuri).collect();
                Ok(vec![(yomi.to_string() + okuri, kanjis)])
            } else {
                // 子音の場合は母音の組み合わせによって全パターンつくって返す。
                let mut result: Vec<(String, Vec<String>)> = Vec::new();
                let yomi_base = &kana[0..kana.len() - last_char.len_utf8()].to_string();
                for boin in self.boin_map.keys() {
                    let Some(okuri) = self
                        .roman_map
                        .get((last_char.to_string() + boin.to_string().as_str()).as_str())
                    else {
                        // "wu" のような、平仮名に変換できない不正なローマ字パターンを生成しているケースもある。
                        // そういう場合は、スキップ。
                        continue;
                    };
                    let kanjis = kanjis.iter().map(|f| f.to_string() + okuri).collect();
                    result.push((yomi_base.to_string() + okuri.to_string().as_str(), kanjis));
                }
                Ok(result)
            }
        } else {
            Ok(vec![(
                kana.to_string(),
                kanjis.iter().map(|f| f.to_string()).collect(),
            )])
        }
    }

    pub fn ari2nasi(
        &self,
        src: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<HashMap<String, Vec<String>>> {
        let mut retval: HashMap<String, Vec<String>> = HashMap::new();
        for (kana, kanjis) in src.iter() {
            for (kkk, vvv) in self.expand_okuri(kana, kanjis)? {
                retval.insert(kkk, vvv);
            }
        }
        Ok(retval)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_expand_okuri() -> anyhow::Result<()> {
        let ari2nasi = Ari2Nasi::default();
        let got = ari2nasi.expand_okuri("あいしあw", &["愛し合".to_string()])?;
        let expected = [
            ("あいしあわ".to_string(), vec!["愛し合わ".to_string()]),
            ("あいしあうぃ".to_string(), vec!["愛し合うぃ".to_string()]),
            ("あいしあうぇ".to_string(), vec!["愛し合うぇ".to_string()]),
            ("あいしあを".to_string(), vec!["愛し合を".to_string()]),
        ];
        assert_eq!(
            got.iter().collect::<HashSet<_>>(),
            expected.iter().collect::<HashSet<_>>(),
        );
        Ok(())
    }

    #[test]
    fn test_expand_okuri_iu() -> anyhow::Result<()> {
        let ari2nasi = Ari2Nasi::default();
        let got = ari2nasi.expand_okuri("いu", &["言".to_string()])?;
        assert_eq!(got, vec![("いう".to_string(), vec!["言う".to_string()])]);
        Ok(())
    }
}
