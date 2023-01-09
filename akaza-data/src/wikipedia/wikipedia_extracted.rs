use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use log::info;
use regex::Regex;
use walkdir::WalkDir;

/// wikiextractor で処理したデータを取り扱うための処理
pub struct ExtractedWikipediaProcessor {
    alnum_pattern: Regex,
    yomigana_pattern: Regex,
}

impl ExtractedWikipediaProcessor {
    pub fn new() -> anyhow::Result<ExtractedWikipediaProcessor> {
        // 英数/記号のみの行を無視するための正規表現。
        // 75||19||colspan=2|-||1||0||76||19
        let alnum_pattern = Regex::new("^[a-zA-Z0-9|=-]+")?;

        // 上級個人情報保護士（じょうきゅうこじんじょうほうほごし）は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
        // → 上級個人情報保護士は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
        let yomigana_pattern = Regex::new(r#"[（\(][\u3041-\u309F、]+[）)]"#)?;

        Ok(ExtractedWikipediaProcessor {
            alnum_pattern,
            yomigana_pattern,
        })
    }

    pub fn process_file<F>(
        &self,
        ifname: &Path,
        ofname: &Path,
        annotate: &mut F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&str) -> anyhow::Result<String>,
    {
        let file = File::open(ifname)?;
        let mut buf = String::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            let line = line.trim();
            if line.starts_with('<') {
                // <doc id="3697757" url="https://ja.wikipedia.org/wiki?curid=3697757"
                //  title="New Sunrise">
                // のような、タグから始まる行を無視する。
                continue;
            }
            if line.is_empty() {
                // 空行を無視する
                continue;
            }
            if self.alnum_pattern.is_match(line) {
                // 英数字のみの行は無視する
                continue;
            }
            let line = self.remove_yomigana(line);

            buf += (annotate(line.as_str()).with_context(|| line)? + "\n").as_str();
        }
        let mut ofile = File::create(ofname)?;
        ofile.write_all(buf.as_bytes())?;
        Ok(())
    }

    fn remove_yomigana(&self, src: &str) -> String {
        self.yomigana_pattern.replace_all(src, "").to_string()
    }

    pub fn get_file_list(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
    ) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
        let mut result: Vec<(PathBuf, PathBuf)> = Vec::new();

        for src_file in WalkDir::new(src_dir)
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| file.metadata().unwrap().is_file())
        {
            let src_path = src_file.path();
            let dirname = src_path.parent().unwrap().file_name().unwrap();
            fs::create_dir_all(dst_dir.join(dirname))?;
            let output_file = dst_dir.join(dirname).join(src_path.file_name().unwrap());

            result.push((src_file.path().to_path_buf(), output_file));
        }
        Ok(result)
    }

    /// ファイルを処理します。
    /// シリアルに処理すると遅いので、パラレルに処理します(する予定)
    pub fn process_files<F>(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        mut annotate: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&str) -> anyhow::Result<String>,
    {
        info!("TODO: Parallel processing");
        for (src_file, dst_file) in self.get_file_list(src_dir, dst_dir)? {
            info!("{} => {}", src_file.display(), dst_file.display());
            self.process_file(src_file.as_path(), &dst_file, &mut annotate)?;
        }

        // _SUCCESS ファイルを書く
        {
            let mut success = File::create(dst_dir.join("_SUCCESS"))?;
            success.write_all("DONE".as_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_yomigana() -> anyhow::Result<()> {
        // 上級個人情報保護士（じょうきゅうこじんじょうほうほごし）は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
        // → 上級個人情報保護士は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
        let runner = ExtractedWikipediaProcessor::new()?;
        let got =
            runner.remove_yomigana("上級個人情報保護士（じょうきゅうこじんじょうほうほごし）は");
        assert_eq!(got, "上級個人情報保護士は");
        Ok(())
    }
}
