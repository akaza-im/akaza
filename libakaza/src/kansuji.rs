use std::cmp::min;

const NUMS: [&str; 10] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
const SUBS: [&str; 4] = ["", "十", "百", "千"];
const PARTS: [&str; 18] = [
    "",
    "万",
    "億",
    "兆",
    "京",
    "垓",
    "𥝱",
    "穣",
    "溝",
    "澗",
    "正",
    "載",
    "極",
    "恒河沙",
    "阿僧祇",
    "那由他",
    "不可思議",
    "無量大数",
];

pub fn int2kanji(i: i64) -> String {
    if i == 0 {
        return "零".to_string();
    }

    let p = i
        .to_string()
        .bytes()
        .map(|b| (b - b'0') as usize)
        .rev()
        .enumerate()
        .collect::<Vec<_>>();
    let mut buf: Vec<&'static str> = Vec::new();
    for &(i, c) in &p {
        if i % 4 == 0 && i > 0 && (i..min(i + 4, p.len())).any(|i| p[i].1 != 0) {
            buf.push(PARTS[i / 4]);
        }
        if c != 0 {
            // その桁が 0 のときは区切りを追加しない
            buf.push(SUBS[i % 4]);
        }
        if !(i % 4 != 0 && c == 1) {
            // 十百千を表示したときで、一のときは追加しない。
            buf.push(NUMS[c]);
        }
    }
    buf.reverse();
    buf.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int2kanji() {
        assert_eq!(int2kanji(0), "零");
        assert_eq!(int2kanji(1), "一");
        assert_eq!(int2kanji(9), "九");
        assert_eq!(int2kanji(10), "十");
        assert_eq!(int2kanji(11), "十一");
        assert_eq!(int2kanji(21), "二十一");
        assert_eq!(int2kanji(99), "九十九");
        assert_eq!(int2kanji(100), "百");
        assert_eq!(int2kanji(999), "九百九十九");
        assert_eq!(int2kanji(1000), "千");
        assert_eq!(int2kanji(9999), "九千九百九十九");
        assert_eq!(int2kanji(10000), "一万");
        assert_eq!(int2kanji(10020), "一万二十");
        assert_eq!(int2kanji(1_000_020), "百万二十");
        assert_eq!(int2kanji(100_000_020), "一億二十");
        assert_eq!(int2kanji(1_0000_4423), "一億四千四百二十三");
        assert_eq!(int2kanji(1_8000_4423), "一億八千万四千四百二十三");
    }
}
