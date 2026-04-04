#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericPrefix {
    pub value: i64,
    pub ascii_digits: String,
    pub consumed_len: usize,
}

struct CounterDef {
    canonical: &'static str,
    aliases: &'static [&'static str],
    surfaces: &'static [&'static str],
}

const COUNTER_DEFS: &[CounterDef] = &[
    CounterDef {
        canonical: "ひき",
        aliases: &["ひき", "びき", "ぴき"],
        surfaces: &["匹"],
    },
    CounterDef {
        canonical: "にん",
        aliases: &["にん"],
        surfaces: &["人"],
    },
    CounterDef {
        canonical: "ほん",
        aliases: &["ほん", "ぼん", "ぽん"],
        surfaces: &["本"],
    },
    CounterDef {
        canonical: "まい",
        aliases: &["まい"],
        surfaces: &["枚"],
    },
    CounterDef {
        canonical: "だい",
        aliases: &["だい"],
        surfaces: &["台"],
    },
    CounterDef {
        canonical: "かい",
        aliases: &["かい"],
        surfaces: &["回", "階"],
    },
    CounterDef {
        canonical: "かいめ",
        aliases: &["かいめ"],
        surfaces: &["回目"],
    },
    CounterDef {
        canonical: "こ",
        aliases: &["こ"],
        surfaces: &["個"],
    },
    CounterDef {
        canonical: "さつ",
        aliases: &["さつ"],
        surfaces: &["冊"],
    },
    CounterDef {
        canonical: "とう",
        aliases: &["とう"],
        surfaces: &["頭"],
    },
    CounterDef {
        canonical: "わ",
        aliases: &["わ"],
        surfaces: &["羽"],
    },
    CounterDef {
        canonical: "ちゃく",
        aliases: &["ちゃく"],
        surfaces: &["着"],
    },
    CounterDef {
        canonical: "けん",
        aliases: &["けん", "げん"],
        surfaces: &["件", "軒"],
    },
    CounterDef {
        canonical: "しゅう",
        aliases: &["しゅう"],
        surfaces: &["週"],
    },
    CounterDef {
        canonical: "しゅうかん",
        aliases: &["しゅうかん"],
        surfaces: &["週間"],
    },
    CounterDef {
        canonical: "ねん",
        aliases: &["ねん"],
        surfaces: &["年"],
    },
    CounterDef {
        canonical: "かげつ",
        aliases: &["かげつ"],
        surfaces: &["か月", "ヶ月", "箇月"],
    },
    CounterDef {
        canonical: "にち",
        aliases: &["にち"],
        surfaces: &["日"],
    },
    CounterDef {
        canonical: "じ",
        aliases: &["じ"],
        surfaces: &["時"],
    },
    CounterDef {
        canonical: "ふん",
        aliases: &["ふん", "ぷん"],
        surfaces: &["分"],
    },
    CounterDef {
        canonical: "びょう",
        aliases: &["びょう"],
        surfaces: &["秒"],
    },
    CounterDef {
        canonical: "さい",
        aliases: &["さい"],
        surfaces: &["歳", "才"],
    },
    CounterDef {
        canonical: "ど",
        aliases: &["ど"],
        surfaces: &["度"],
    },
    CounterDef {
        canonical: "ばん",
        aliases: &["ばん"],
        surfaces: &["番"],
    },
    CounterDef {
        canonical: "えん",
        aliases: &["えん"],
        surfaces: &["円"],
    },
    CounterDef {
        canonical: "じかん",
        aliases: &["じかん"],
        surfaces: &["時間"],
    },
    CounterDef {
        canonical: "にちかん",
        aliases: &["にちかん"],
        surfaces: &["日間"],
    },
    CounterDef {
        canonical: "はい",
        aliases: &["はい", "ぱい"],
        surfaces: &["杯"],
    },
    CounterDef {
        canonical: "ばい",
        aliases: &["ばい"],
        surfaces: &["倍", "杯"],
    },
    CounterDef {
        canonical: "きょく",
        aliases: &["きょく"],
        surfaces: &["曲"],
    },
    CounterDef {
        canonical: "もん",
        aliases: &["もん"],
        surfaces: &["問"],
    },
    CounterDef {
        canonical: "めい",
        aliases: &["めい"],
        surfaces: &["名"],
    },
    CounterDef {
        canonical: "てん",
        aliases: &["てん"],
        surfaces: &["点"],
    },
    CounterDef {
        canonical: "ひょう",
        aliases: &["ひょう"],
        surfaces: &["票"],
    },
    CounterDef {
        canonical: "つう",
        aliases: &["つう"],
        surfaces: &["通"],
    },
    CounterDef {
        canonical: "しょく",
        aliases: &["しょく"],
        surfaces: &["食"],
    },
    CounterDef {
        canonical: "わり",
        aliases: &["わり"],
        surfaces: &["割"],
    },
    CounterDef {
        canonical: "はく",
        aliases: &["はく", "ぱく"],
        surfaces: &["泊"],
    },
    CounterDef {
        canonical: "はつ",
        aliases: &["はつ", "ぱつ"],
        surfaces: &["発"],
    },
];

/// COUNTER_DEFS から全エイリアスを集約した配列を返す。
/// 手動同期の不整合を防ぐため、COUNTER_DEFS を唯一の情報源とする。
fn collect_counter_yomi_aliases() -> Vec<&'static str> {
    let mut aliases = Vec::new();
    for def in COUNTER_DEFS {
        for alias in def.aliases {
            if !aliases.contains(alias) {
                aliases.push(alias);
            }
        }
    }
    aliases
}

const KANA_THOUSANDS: [(&str, i64); 10] = [
    ("きゅうせん", 9000),
    ("はっせん", 8000),
    ("ななせん", 7000),
    ("ろくせん", 6000),
    ("ごせん", 5000),
    ("よんせん", 4000),
    ("さんぜん", 3000),
    ("にせん", 2000),
    ("いっせん", 1000),
    ("せん", 1000),
];

const KANA_HUNDREDS: [(&str, i64); 19] = [
    ("きゅうひゃく", 900),
    ("きゅうひゃっ", 900),
    ("はっぴゃく", 800),
    ("はっぴゃっ", 800),
    ("ななひゃく", 700),
    ("ななひゃっ", 700),
    ("ろっぴゃく", 600),
    ("ろっぴゃっ", 600),
    ("ごひゃく", 500),
    ("ごひゃっ", 500),
    ("よんひゃく", 400),
    ("よんひゃっ", 400),
    ("さんびゃく", 300),
    ("さんびゃっ", 300),
    ("にひゃく", 200),
    ("にひゃっ", 200),
    ("いっぴゃく", 100),
    ("ひゃく", 100),
    ("ひゃっ", 100),
];

const KANA_TENS: [(&str, i64); 11] = [
    ("きゅうじゅう", 90),
    ("はちじゅう", 80),
    ("ななじゅう", 70),
    ("ろくじゅう", 60),
    ("ごじゅう", 50),
    ("よんじゅう", 40),
    ("さんじゅう", 30),
    ("にじゅう", 20),
    ("いちじゅう", 10),
    ("じゅう", 10),
    ("じゅっ", 10),
];

const KANA_ONES: [(&str, i64); 13] = [
    ("ぜろ", 0),
    ("れい", 0),
    ("きゅう", 9),
    ("はち", 8),
    ("なな", 7),
    ("しち", 7),
    ("ろく", 6),
    ("ご", 5),
    ("よん", 4),
    ("さん", 3),
    ("に", 2),
    ("いち", 1),
    ("いっ", 1),
];

fn fullwidth_to_ascii(ch: char) -> Option<char> {
    if ('０'..='９').contains(&ch) {
        Some(char::from_u32((ch as u32) - ('０' as u32) + ('0' as u32)).unwrap())
    } else {
        None
    }
}

pub fn to_fullwidth_digits(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                char::from_u32((c as u32) - ('0' as u32) + ('０' as u32)).unwrap()
            } else {
                c
            }
        })
        .collect()
}

fn kanji_digit_value(ch: char) -> Option<i64> {
    match ch {
        '零' | '〇' => Some(0),
        '一' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

fn kanji_small_unit_value(ch: char) -> Option<i64> {
    match ch {
        '十' => Some(10),
        '百' => Some(100),
        '千' => Some(1000),
        _ => None,
    }
}

fn kanji_large_unit_value(ch: char) -> Option<i64> {
    match ch {
        '万' => Some(10_i64.pow(4)),
        '億' => Some(10_i64.pow(8)),
        '兆' => Some(10_i64.pow(12)),
        '京' => Some(10_i64.pow(16)),
        _ => None,
    }
}

fn is_kanji_numeral_char(ch: char) -> bool {
    kanji_digit_value(ch).is_some()
        || kanji_small_unit_value(ch).is_some()
        || kanji_large_unit_value(ch).is_some()
}

fn parse_ascii_or_fullwidth_digits_prefix(s: &str) -> Option<NumericPrefix> {
    let mut ascii = String::new();
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if ch.is_ascii_digit() {
            ascii.push(ch);
            end = idx + ch.len_utf8();
        } else if let Some(a) = fullwidth_to_ascii(ch) {
            ascii.push(a);
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if ascii.is_empty() {
        return None;
    }
    let value = ascii.parse::<i64>().ok()?;
    Some(NumericPrefix {
        value,
        ascii_digits: ascii,
        consumed_len: end,
    })
}

fn parse_kanji_number_exact(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.iter().any(|ch| !is_kanji_numeral_char(*ch)) {
        return None;
    }

    let has_unit = chars
        .iter()
        .any(|ch| kanji_small_unit_value(*ch).is_some() || kanji_large_unit_value(*ch).is_some());
    if !has_unit {
        let mut value: i64 = 0;
        for ch in chars {
            let d = kanji_digit_value(ch)?;
            value = value.checked_mul(10)?.checked_add(d)?;
        }
        return Some(value);
    }

    let mut total: i128 = 0;
    let mut section: i128 = 0;
    let mut digit: i128 = 0;
    for ch in chars {
        if let Some(d) = kanji_digit_value(ch) {
            digit = i128::from(d);
            continue;
        }
        if let Some(u) = kanji_small_unit_value(ch) {
            let base = if digit == 0 { 1 } else { digit };
            section += base * i128::from(u);
            digit = 0;
            continue;
        }
        if let Some(u) = kanji_large_unit_value(ch) {
            let base = section + digit;
            let block = if base == 0 { 1 } else { base };
            total += block * i128::from(u);
            section = 0;
            digit = 0;
            continue;
        }
        return None;
    }
    total += section + digit;
    i64::try_from(total).ok()
}

fn parse_kanji_number_prefix(s: &str) -> Option<NumericPrefix> {
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if is_kanji_numeral_char(ch) {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }

    let mut consumed = end;
    while consumed > 0 {
        let prefix = &s[..consumed];
        if let Some(value) = parse_kanji_number_exact(prefix) {
            return Some(NumericPrefix {
                value,
                ascii_digits: value.to_string(),
                consumed_len: consumed,
            });
        }
        let mut prev = 0;
        for (idx, _) in prefix.char_indices() {
            prev = idx;
        }
        consumed = prev;
    }
    None
}

fn longest_match<'a>(s: &'a str, table: &[(&'a str, i64)]) -> Option<(usize, i64)> {
    let mut best: Option<(usize, i64)> = None;
    for (tok, value) in table {
        if s.starts_with(tok) {
            let len = tok.len();
            if best.map(|(l, _)| len > l).unwrap_or(true) {
                best = Some((len, *value));
            }
        }
    }
    best
}

fn parse_kana_under_10000_exact(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut rest = s;
    let mut value: i64 = 0;
    let mut consumed_any = false;

    for table in [&KANA_THOUSANDS[..], &KANA_HUNDREDS[..], &KANA_TENS[..]] {
        if let Some((len, v)) = longest_match(rest, table) {
            value += v;
            rest = &rest[len..];
            consumed_any = true;
        }
    }
    if let Some((len, v)) = longest_match(rest, &KANA_ONES) {
        value += v;
        rest = &rest[len..];
        consumed_any = true;
    }
    if !rest.is_empty() || !consumed_any {
        return None;
    }
    Some(value)
}

fn parse_kana_number_exact(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut rest = s;
    let mut total: i128 = 0;
    let mut consumed_large = false;
    for (unit_word, unit_value) in [
        ("ちょう", 10_i64.pow(12)),
        ("おく", 10_i64.pow(8)),
        ("まん", 10_i64.pow(4)),
    ] {
        if let Some(pos) = rest.find(unit_word) {
            let block = &rest[..pos];
            let block_value = if block.is_empty() {
                1
            } else {
                parse_kana_under_10000_exact(block)?
            };
            total += i128::from(block_value) * i128::from(unit_value);
            rest = &rest[pos + unit_word.len()..];
            consumed_large = true;
        }
    }

    if !rest.is_empty() {
        total += i128::from(parse_kana_under_10000_exact(rest)?);
    } else if !consumed_large {
        return None;
    }

    i64::try_from(total).ok()
}

pub fn parse_numeric_prefix_surface(s: &str) -> Option<NumericPrefix> {
    parse_ascii_or_fullwidth_digits_prefix(s).or_else(|| parse_kanji_number_prefix(s))
}

pub fn parse_numeric_exact_reading(s: &str) -> Option<i64> {
    if let Some(p) = parse_ascii_or_fullwidth_digits_prefix(s) {
        if p.consumed_len == s.len() {
            return Some(p.value);
        }
    }
    if let Some(p) = parse_kanji_number_prefix(s) {
        if p.consumed_len == s.len() {
            return Some(p.value);
        }
    }
    parse_kana_number_exact(s)
}

/// 1文字かな数詞（に, し, ご, く）は助詞や一般語と衝突するため除外対象とする。
/// 例: 「にほん」→「2+本」と誤解析するのを防ぐ。
const AMBIGUOUS_SINGLE_CHAR_NUMERALS: &[&str] = &["に", "ご"];

/// かな数詞パスでの助数詞マッチに使う最小文字数。
/// 1文字助数詞（じ=時, ど=度, こ=個, わ=羽）は曖昧性が高すぎるため、
/// かな数詞との組み合わせでは除外する。ASCII/全角数字との組み合わせでは有効。
const MIN_COUNTER_LEN_FOR_KANA_NUMERIC: usize = 2;

pub fn parse_kana_numeric_prefix_before_counter(s: &str) -> Option<NumericPrefix> {
    let mut best: Option<NumericPrefix> = None;
    for (split, _) in s.char_indices().skip(1) {
        let prefix = &s[..split];
        let suffix = &s[split..];
        // 助数詞が suffix に完全一致するケースのみマッチする。
        // starts_with だと「じょう」が「じ」(時) にマッチするような誤爆が起きる。
        let matched_counter = counter_yomi_aliases()
            .iter()
            .any(|a| suffix == *a && a.chars().count() >= MIN_COUNTER_LEN_FOR_KANA_NUMERIC);
        if !matched_counter {
            continue;
        }
        // 1文字かな数詞のみの場合は誤爆リスクが高いため除外
        if AMBIGUOUS_SINGLE_CHAR_NUMERALS.contains(&prefix) {
            continue;
        }
        if let Some(value) = parse_kana_number_exact(prefix) {
            best = Some(NumericPrefix {
                value,
                ascii_digits: value.to_string(),
                consumed_len: split,
            });
        }
    }
    best
}

pub fn counter_yomi_aliases() -> &'static [&'static str] {
    use std::sync::OnceLock;
    static ALIASES: OnceLock<Vec<&'static str>> = OnceLock::new();
    ALIASES.get_or_init(collect_counter_yomi_aliases)
}

pub fn normalize_counter_yomi(yomi: &str) -> Option<&'static str> {
    for def in COUNTER_DEFS {
        if def.aliases.contains(&yomi) {
            return Some(def.canonical);
        }
    }
    None
}

pub fn counter_surfaces_for(canonical_yomi: &str) -> Option<&'static [&'static str]> {
    for def in COUNTER_DEFS {
        if def.canonical == canonical_yomi {
            return Some(def.surfaces);
        }
    }
    None
}

/// 助数詞の user 学習を数字に依存しない形で集約するためのキー正規化。
pub fn normalize_counter_key_for_lm(key: &str) -> Option<String> {
    let slash_pos = key.find('/')?;
    let surface = &key[..slash_pos];
    let reading = &key[slash_pos + 1..];

    let surface_prefix = parse_numeric_prefix_surface(surface)?;
    if surface_prefix.consumed_len >= surface.len() {
        return None;
    }
    let surface_suffix = &surface[surface_prefix.consumed_len..];

    let mut canonical_yomi: Option<&'static str> = None;
    for alias in counter_yomi_aliases() {
        if let Some(num_reading) = reading.strip_suffix(alias) {
            if num_reading.is_empty() {
                continue;
            }
            if parse_numeric_exact_reading(num_reading).is_some() {
                canonical_yomi = normalize_counter_yomi(alias);
                if canonical_yomi.is_some() {
                    break;
                }
            }
        }
    }
    let canonical_yomi = canonical_yomi?;

    let allowed_surfaces = counter_surfaces_for(canonical_yomi)?;
    if !allowed_surfaces.contains(&surface_suffix) {
        return None;
    }
    Some(format!("<NUM>{surface_suffix}/<NUM>{canonical_yomi}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_prefix_surface() {
        assert_eq!(
            parse_numeric_prefix_surface("５１６週間"),
            Some(NumericPrefix {
                value: 516,
                ascii_digits: "516".to_string(),
                consumed_len: 9
            })
        );
        assert_eq!(
            parse_numeric_prefix_surface("五百十六週間"),
            Some(NumericPrefix {
                value: 516,
                ascii_digits: "516".to_string(),
                consumed_len: "五百十六".len()
            })
        );
    }

    #[test]
    fn test_parse_kana_numeric_prefix_before_counter() {
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("ひゃくさんびき"),
            Some(NumericPrefix {
                value: 103,
                ascii_digits: "103".to_string(),
                consumed_len: "ひゃくさん".len()
            })
        );
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("さんびき"),
            Some(NumericPrefix {
                value: 3,
                ascii_digits: "3".to_string(),
                consumed_len: "さん".len()
            })
        );
    }

    #[test]
    fn test_parse_numeric_exact_reading() {
        assert_eq!(parse_numeric_exact_reading("516"), Some(516));
        assert_eq!(parse_numeric_exact_reading("５１６"), Some(516));
        assert_eq!(parse_numeric_exact_reading("五百十六"), Some(516));
        assert_eq!(parse_numeric_exact_reading("ごひゃくじゅうろく"), Some(516));
    }

    #[test]
    fn test_normalize_counter_key_for_lm() {
        assert_eq!(
            normalize_counter_key_for_lm("3匹/3びき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("５１６週間/516しゅうかん"),
            Some("<NUM>週間/<NUM>しゅうかん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("五百十六週間/ごひゃくじゅうろくしゅうかん"),
            Some("<NUM>週間/<NUM>しゅうかん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("0匹/ぜろひき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("3本/3ぼん"),
            Some("<NUM>本/<NUM>ほん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("１０分/じゅっぷん"),
            Some("<NUM>分/<NUM>ふん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("3人/3にん"),
            Some("<NUM>人/<NUM>にん".to_string())
        );
    }

    /// COUNTER_DEFS の全エイリアスが counter_yomi_aliases() に含まれていることを検証する。
    #[test]
    fn test_counter_yomi_aliases_matches_counter_defs() {
        let aliases = counter_yomi_aliases();
        for def in COUNTER_DEFS {
            for alias in def.aliases {
                assert!(
                    aliases.contains(alias),
                    "alias {:?} from COUNTER_DEFS (canonical={:?}) is missing in counter_yomi_aliases()",
                    alias,
                    def.canonical
                );
            }
        }
    }

    /// 1文字かな数詞が助詞・一般語と衝突しないことを検証する。
    /// 「にほん」→「2+本」、「ごにん」→「5+にん」等の誤爆を防ぐ。
    #[test]
    fn test_kana_numeral_no_false_positive_on_common_words() {
        // 「にほん」は「日本」であり「2本」ではない
        assert_eq!(parse_kana_numeric_prefix_before_counter("にほん"), None);
        // 「ごけん」は「語研/ご件」と曖昧だが1文字数詞なのでブロック
        assert_eq!(parse_kana_numeric_prefix_before_counter("ごけん"), None);

        // 「く」「し」は KANA_ONES に含まれないため、そもそも数値として解釈されない
        assert_eq!(parse_kana_numeric_prefix_before_counter("くにん"), None);
        assert_eq!(parse_kana_numeric_prefix_before_counter("しまい"), None);

        // 一方、2文字以上のかな数詞は正しく動作する
        assert!(parse_kana_numeric_prefix_before_counter("さんびき").is_some()); // 3匹
        assert!(parse_kana_numeric_prefix_before_counter("ぜろほん").is_some()); // 0本
        assert!(parse_kana_numeric_prefix_before_counter("じゅっぷん").is_some()); // 10分
        assert!(parse_kana_numeric_prefix_before_counter("にせんえん").is_some());
        // 2000円
    }

    /// 「く」(9) と「し」(4) はかな数詞パーサーで数値として扱わない。
    /// これらは固定熟語（くがつ, しがつ等）でのみ使われ、
    /// 数の読みとしては「きゅう」「よん」が標準。
    /// 「くちょう」→9兆、「しちょう」→4兆のような誤変換を防ぐ。
    #[test]
    fn test_ku_shi_not_treated_as_numbers() {
        // 「く」「し」は parse_kana_number_exact で数値にならない
        assert_eq!(parse_numeric_exact_reading("くちょう"), None); // 口調/区長であり9兆ではない
        assert_eq!(parse_numeric_exact_reading("くおく"), None); // 9億ではない
        assert_eq!(parse_numeric_exact_reading("くまん"), None); // 9万ではない
        assert_eq!(parse_numeric_exact_reading("しちょう"), None); // 市長/視聴であり4兆ではない
        assert_eq!(parse_numeric_exact_reading("しおく"), None); // 4億ではない
        assert_eq!(parse_numeric_exact_reading("しまん"), None); // 4万ではない
        assert_eq!(parse_numeric_exact_reading("しじゅう"), None); // 始終であり40ではない

        // 「きゅう」「よん」は標準的な読みなので正しく動作する
        assert_eq!(
            parse_numeric_exact_reading("きゅうちょう"),
            Some(9_000_000_000_000)
        );
        assert_eq!(parse_numeric_exact_reading("きゅうおく"), Some(900_000_000));
        assert_eq!(parse_numeric_exact_reading("きゅうまん"), Some(90_000));
        assert_eq!(
            parse_numeric_exact_reading("よんちょう"),
            Some(4_000_000_000_000)
        );
        assert_eq!(parse_numeric_exact_reading("よんおく"), Some(400_000_000));
        assert_eq!(parse_numeric_exact_reading("よんまん"), Some(40_000));
        assert_eq!(parse_numeric_exact_reading("よんじゅう"), Some(40));

        // 「に」「ご」は標準的な読みなので引き続き動作する
        assert_eq!(parse_numeric_exact_reading("にまん"), Some(20_000));
        assert_eq!(parse_numeric_exact_reading("ごまん"), Some(50_000));
        assert_eq!(parse_numeric_exact_reading("におく"), Some(200_000_000));
        assert_eq!(parse_numeric_exact_reading("ごおく"), Some(500_000_000));
    }

    /// 「く」「し」を含む助数詞パターンが数字に変換されないことを検証する。
    #[test]
    fn test_ku_shi_before_counter_not_numeric() {
        // 「くちょうえん」は「口調+円」等であり「9兆円」ではない
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("くちょうえん"),
            None
        );
        // 「しちょうえん」は「市長+円」等であり「4兆円」ではない
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("しちょうえん"),
            None
        );

        // 標準的な読みでは正しく動作する
        assert!(parse_kana_numeric_prefix_before_counter("きゅうちょうえん").is_some()); // 9兆円
        assert!(parse_kana_numeric_prefix_before_counter("よんちょうえん").is_some());
        // 4兆円
    }

    /// 促音便を含むかな数詞+助数詞が正しくパースされることを検証する。
    /// 例: さんびゃっぴき (300匹), ろっぴゃっぽん (600本)
    #[test]
    fn test_kana_numeric_sokuonbin_hundreds() {
        // さんびゃっぴき = 300匹
        let result = parse_kana_numeric_prefix_before_counter("さんびゃっぴき");
        assert_eq!(
            result,
            Some(NumericPrefix {
                value: 300,
                ascii_digits: "300".to_string(),
                consumed_len: "さんびゃっ".len()
            })
        );

        // ろっぴゃっぴき = 600匹
        let result = parse_kana_numeric_prefix_before_counter("ろっぴゃっぴき");
        assert_eq!(
            result,
            Some(NumericPrefix {
                value: 600,
                ascii_digits: "600".to_string(),
                consumed_len: "ろっぴゃっ".len()
            })
        );

        // はっぴゃっぴき = 800匹
        let result = parse_kana_numeric_prefix_before_counter("はっぴゃっぴき");
        assert_eq!(
            result,
            Some(NumericPrefix {
                value: 800,
                ascii_digits: "800".to_string(),
                consumed_len: "はっぴゃっ".len()
            })
        );

        // にひゃっぴき = 200匹
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("にひゃっぴき"),
            Some(NumericPrefix {
                value: 200,
                ascii_digits: "200".to_string(),
                consumed_len: "にひゃっ".len()
            })
        );

        // よんひゃっぴき = 400匹
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("よんひゃっぴき"),
            Some(NumericPrefix {
                value: 400,
                ascii_digits: "400".to_string(),
                consumed_len: "よんひゃっ".len()
            })
        );

        // ごひゃっぴき = 500匹
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("ごひゃっぴき"),
            Some(NumericPrefix {
                value: 500,
                ascii_digits: "500".to_string(),
                consumed_len: "ごひゃっ".len()
            })
        );

        // ななひゃっぴき = 700匹
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("ななひゃっぴき"),
            Some(NumericPrefix {
                value: 700,
                ascii_digits: "700".to_string(),
                consumed_len: "ななひゃっ".len()
            })
        );

        // きゅうひゃっぴき = 900匹
        assert_eq!(
            parse_kana_numeric_prefix_before_counter("きゅうひゃっぴき"),
            Some(NumericPrefix {
                value: 900,
                ascii_digits: "900".to_string(),
                consumed_len: "きゅうひゃっ".len()
            })
        );
    }
}
