use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use log::{info, warn};

/// Check if a string contains at least one Japanese character
/// (hiragana, katakana, CJK unified ideographs, or CJK extension A).
fn contains_japanese(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c,
            '\u{3040}'..='\u{309F}'   // Hiragana
            | '\u{30A0}'..='\u{30FF}' // Katakana
            | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
            | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        )
    })
}

/// 小文字かな（拗音・促音）で始まるかどうかを判定する。
/// vibrato の分節ミスで生まれる不正なトークンを弾くために使う。
fn starts_with_small_kana(s: &str) -> bool {
    let Some(c) = s.chars().next() else {
        return false;
    };
    matches!(
        c,
        'ぁ' | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'っ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ッ'
    )
}

/// wfreq (単語の発生頻度表)から vocab (語彙ファイル)を作成する。
pub fn vocab(src_file: &str, dst_file: &str, threshold: u32) -> anyhow::Result<()> {
    info!(
        "vocab: {} => {}, threshold={}",
        src_file, dst_file, threshold
    );

    let ifp = File::open(src_file)?;
    let mut ofp = File::create(dst_file.to_string() + ".tmp")?;
    for line in BufReader::new(ifp).lines() {
        let line = line?;
        let line = line.trim();
        let Some((word, cnt)) = line.split_once('\t') else {
            warn!("Skipping malformed wfreq line: {:?}", line);
            continue;
        };
        if word.starts_with(' ') || word.starts_with('/') {
            warn!("Invalid word: {:?}", line);
            continue;
        }
        if !word.contains('/') {
            warn!("Invalid word: {:?}", line);
            continue;
        }
        let surface = word.split('/').next().unwrap_or("");
        if !contains_japanese(surface) {
            warn!("Skipping non-Japanese surface: {:?}", word);
            continue;
        }
        if starts_with_small_kana(surface) {
            warn!("Skipping small-kana-initial word: {:?}", word);
            continue;
        }
        let cnt: u32 = cnt.parse()?;
        if cnt > threshold {
            ofp.write_fmt(format_args!("{word}\n"))?;
        }
    }
    fs::rename(dst_file.to_owned() + ".tmp", dst_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_contains_japanese_hiragana() {
        assert!(contains_japanese("あ"));
        assert!(contains_japanese("こんにちは"));
    }

    #[test]
    fn test_contains_japanese_katakana() {
        assert!(contains_japanese("ア"));
        assert!(contains_japanese("カタカナ"));
    }

    #[test]
    fn test_contains_japanese_kanji() {
        assert!(contains_japanese("漢"));
        assert!(contains_japanese("東京"));
    }

    #[test]
    fn test_contains_japanese_cjk_ext_a() {
        // U+3400 is in CJK Unified Ideographs Extension A
        assert!(contains_japanese("\u{3400}"));
    }

    #[test]
    fn test_contains_japanese_mixed() {
        assert!(contains_japanese("hello世界"));
        assert!(contains_japanese("123あ456"));
    }

    #[test]
    fn test_contains_japanese_pure_ascii() {
        assert!(!contains_japanese("hello"));
        assert!(!contains_japanese("!!!!!"));
        assert!(!contains_japanese("http"));
        assert!(!contains_japanese("(^_^;)"));
        assert!(!contains_japanese("(-_-;)"));
    }

    #[test]
    fn test_contains_japanese_emoji_only() {
        assert!(!contains_japanese("😣"));
        assert!(!contains_japanese("🎉🎊"));
    }

    #[test]
    fn test_contains_japanese_symbols_only() {
        assert!(!contains_japanese("⇩"));
        assert!(!contains_japanese("✓"));
        assert!(!contains_japanese("→←↑↓"));
    }

    #[test]
    fn test_contains_japanese_empty() {
        assert!(!contains_japanese(""));
    }

    #[test]
    fn test_vocab_filters_non_japanese_surface() {
        let mut src = NamedTempFile::new().unwrap();
        // Japanese surface: should be included (cnt > threshold=0)
        writeln!(src, "東京/トウキョウ\t10").unwrap();
        // Pure ASCII surface: should be filtered
        writeln!(src, "!!!!!/キゴウ\t10").unwrap();
        // Emoticon surface: should be filtered
        writeln!(src, "(^_^;)/カオモジ\t10").unwrap();
        // Emoji surface: should be filtered
        writeln!(src, "😣/エモジ\t10").unwrap();
        // Katakana surface: should be included
        writeln!(src, "カタカナ/カタカナ\t10").unwrap();
        // Hiragana surface: should be included
        writeln!(src, "ひらがな/ヒラガナ\t10").unwrap();
        src.flush().unwrap();

        let dst = NamedTempFile::new().unwrap();
        let dst_path = dst.path().to_str().unwrap().to_string();

        vocab(src.path().to_str().unwrap(), &dst_path, 0).unwrap();

        let result = fs::read_to_string(&dst_path).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "東京/トウキョウ");
        assert_eq!(lines[1], "カタカナ/カタカナ");
        assert_eq!(lines[2], "ひらがな/ヒラガナ");
    }

    #[test]
    fn test_starts_with_small_kana() {
        assert!(starts_with_small_kana("ゅうらんしゃじけん"));
        assert!(starts_with_small_kana("ッテ"));
        assert!(starts_with_small_kana("ぁいう"));
        assert!(!starts_with_small_kana("あいう"));
        assert!(!starts_with_small_kana("カタカナ"));
        assert!(!starts_with_small_kana("漢字"));
        assert!(!starts_with_small_kana(""));
    }

    #[test]
    fn test_vocab_filters_small_kana_initial() {
        let mut src = NamedTempFile::new().unwrap();
        writeln!(src, "東京/とうきょう\t10").unwrap();
        writeln!(src, "ゅうらんしゃじけん/ゅうらんしゃじけん\t10").unwrap();
        writeln!(src, "ッテ/って\t10").unwrap();
        src.flush().unwrap();

        let dst = NamedTempFile::new().unwrap();
        let dst_path = dst.path().to_str().unwrap().to_string();

        vocab(src.path().to_str().unwrap(), &dst_path, 0).unwrap();

        let result = fs::read_to_string(&dst_path).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "東京/とうきょう");
    }

    #[test]
    fn test_vocab_respects_threshold() {
        let mut src = NamedTempFile::new().unwrap();
        writeln!(src, "東京/トウキョウ\t100").unwrap();
        writeln!(src, "大阪/オオサカ\t5").unwrap();
        src.flush().unwrap();

        let dst = NamedTempFile::new().unwrap();
        let dst_path = dst.path().to_str().unwrap().to_string();

        vocab(src.path().to_str().unwrap(), &dst_path, 10).unwrap();

        let result = fs::read_to_string(&dst_path).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "東京/トウキョウ");
    }
}
