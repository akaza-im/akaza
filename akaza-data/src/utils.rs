use std::borrow::Cow;

use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// `"path:weight"` 形式の文字列をパースする。
/// weight が省略された場合は 1.0 を返す。
///
/// 例: `"work/jawiki/:0.3"` → `("work/jawiki/", 0.3)`
///     `"work/jawiki/"` → `("work/jawiki/", 1.0)`
pub fn parse_dir_weight(s: &str) -> (String, f64) {
    if let Some(pos) = s.rfind(':') {
        let (path, weight_str) = s.split_at(pos);
        if let Ok(w) = weight_str[1..].parse::<f64>() {
            return (path.to_string(), w);
        }
    }
    (s.to_string(), 1.0)
}

pub fn get_file_list(src_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = Vec::new();

    for src_file in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
    {
        result.push(src_file.path().to_path_buf());
    }
    Ok(result)
}

/// 数字+接尾辞トークンを `<NUM>` に正規化する。
///
/// 裸の数字（suffix なし）は正規化しない。全数字カウントが集約されると
/// `<NUM>/<NUM>` のスコアが極端に高くなり、助詞「に」「さん」「ご」等が
/// 数字に化ける退行を引き起こすため。
///
/// - `"1匹/1ひき"` → `"<NUM>匹/<NUM>匹"`
/// - `"1/1"` → `"1/1"` (変換なし — 裸の数字)
/// - `"匹/ひき"` → `"匹/ひき"` (変換なし)
pub fn normalize_num_token(word: &str) -> Cow<'_, str> {
    let Some(slash_pos) = word.find('/') else {
        return Cow::Borrowed(word);
    };
    let surface = &word[..slash_pos];
    let digit_end = surface.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digit_end == 0 {
        return Cow::Borrowed(word);
    }
    let suffix = &surface[digit_end..];
    if suffix.is_empty() {
        // 裸の数字は正規化しない
        Cow::Borrowed(word)
    } else {
        Cow::Owned(format!("<NUM>{0}/<NUM>{0}", suffix))
    }
}

pub fn copy_snapshot(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all("work/dump/")?;
    fs::copy(
        path,
        Path::new("work/dump/").join(
            Local::now().format("%Y%m%d-%H%M%S").to_string()
                + path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .as_str(),
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dir_weight_with_weight() {
        let (path, weight) = parse_dir_weight("work/jawiki/:0.3");
        assert_eq!(path, "work/jawiki/");
        assert!((weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_dir_weight_without_weight() {
        let (path, weight) = parse_dir_weight("work/jawiki/");
        assert_eq!(path, "work/jawiki/");
        assert!((weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_normalize_num_token_with_suffix() {
        assert_eq!(normalize_num_token("1匹/1ひき"), "<NUM>匹/<NUM>匹");
    }

    #[test]
    fn test_normalize_num_token_with_suffix_yen() {
        assert_eq!(normalize_num_token("100円/100えん"), "<NUM>円/<NUM>円");
    }

    #[test]
    fn test_normalize_num_token_digit_only() {
        // 裸の数字は正規化しない（スコア集約による退行を防止）
        assert_eq!(normalize_num_token("1/1"), "1/1");
    }

    #[test]
    fn test_normalize_num_token_no_digit() {
        assert_eq!(normalize_num_token("匹/ひき"), "匹/ひき");
    }

    #[test]
    fn test_normalize_num_token_non_leading_digit() {
        // 「第」で始まるので変換なし
        assert_eq!(normalize_num_token("第1回/だい1かい"), "第1回/だい1かい");
    }

    #[test]
    fn test_normalize_num_token_year() {
        assert_eq!(normalize_num_token("2019年/2019ねん"), "<NUM>年/<NUM>年");
    }

    #[test]
    fn test_parse_dir_weight_with_colon_in_path() {
        // Windows-style path like "C:/Users/foo" should not be parsed as weight
        let (path, weight) = parse_dir_weight("C:/Users/foo");
        assert_eq!(path, "C:/Users/foo");
        assert!((weight - 1.0).abs() < f64::EPSILON);
    }
}
