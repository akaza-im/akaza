use encoding_rs::SHIFT_JIS;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::Context;
use log::info;

use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

/// wikiextractor で処理したデータを取り扱うための処理
pub struct AozoraBunkoProcessor {
    alnum_pattern: Regex,
    yomigana_pattern: Regex,
    comment_pattern: Regex,
    kyukana_pattern: Regex,
    meta_separator_pattern: Regex,
    sokohon_pattern: Regex,
}

impl AozoraBunkoProcessor {
    pub fn new() -> anyhow::Result<AozoraBunkoProcessor> {
        // 英数/記号のみの行を無視するための正規表現。
        // 75||19||colspan=2|-||1||0||76||19
        let alnum_pattern = Regex::new("^[a-zA-Z0-9|=-]+")?;

        // 小《ちひ》さな
        // のようなよみがなを無視する。
        let yomigana_pattern = Regex::new(r#"《.*?》"#)?;

        // コメントのパターン。
        let comment_pattern = Regex::new("［＃.*］")?;

        // 旧仮名遣いのパターン。
        let kyukana_pattern = Regex::new("[ゐヰゑヱ]")?;

        let meta_separator_pattern = RegexBuilder::new(".*-{10,}\r?\n")
            .dot_matches_new_line(true)
            .build()?;

        let sokohon_pattern = RegexBuilder::new("底本：.*")
            .dot_matches_new_line(true)
            .build()?;

        Ok(AozoraBunkoProcessor {
            alnum_pattern,
            yomigana_pattern,
            comment_pattern,
            kyukana_pattern,
            meta_separator_pattern,
            sokohon_pattern,
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
        let mut file = File::open(ifname)?;
        let mut vec_buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut vec_buf)?;
        let (src, _, _) = SHIFT_JIS.decode(&vec_buf);
        let src = src.replace('\r', "");

        // _ruby_ がパスに含まれている場合は、ルビが振られているので古い文書だと思う。
        if ifname.to_string_lossy().contains("_ruby_") {
            info!("Skipping {} due to _ruby_", ifname.to_string_lossy());
            return Ok(());
        }

        // 二倍の踊り字 についての表記がある場合、旧仮名遣いであることが多い。
        // ので、かな漢字変換用のコーパスとしては不適切なので無視する。
        if src.contains("二倍の踊り字") {
            info!("Skipping {} due to 二倍の踊り字", ifname.to_string_lossy());
            return Ok(());
        }

        // 第3水準の文字が含まれている文書の場合、文書として独特すぎるケースが多いので
        // 第3水準の文字が含まれるファイルは無視する。
        //
        // https://www.aozora.gr.jp/cards/000712/files/52341_42513.html
        //（例）※［＃「くさかんむり／孚」、第3水準1-90-90］
        if src.contains("第3水準") {
            info!("Skipping {} due to 第3水準", ifname.to_string_lossy());
            return Ok(());
        }

        if src.contains("creativecommons.org") {
            info!(
                "Skipping {} due to creativecommons.org",
                ifname.to_string_lossy()
            );
            return Ok(());
        }

        // 「旧字、旧仮名で書かれた作品を、現代表記にあらためる際の作業指針」
        // について言及している文書はスキップする。
        //
        // https://www.aozora.gr.jp/cards/000712/files/52341_42513.html
        if src.contains("旧字、旧仮名で書かれた作品を、現代表記にあらためる際の作業指針")
        {
            info!(
                "Skipping {} due to 旧字、旧仮名で書かれた作品を、現代表記にあらためる際の作業指針",
                ifname.to_string_lossy()
            );
            return Ok(());
        }

        // 明らかな旧仮名遣いを検出する
        if src.contains("旧字、旧仮名で書かれた作品を、現代表記にあらためる際の作業指針")
        {
            info!(
                "Skipping {} due to 旧字、旧仮名で書かれた作品を、現代表記にあらためる際の作業指針",
                ifname.to_string_lossy()
            );
            return Ok(());
        }

        if self.is_kyukana(src.as_str()) {
            info!("Skipping {} due to 旧仮名", ifname.to_string_lossy());
            return Ok(());
        }

        let src = self.strip_meta(src.as_str());

        let mut buf = String::new();
        for line in src.lines() {
            let line = line.trim();
            if line.starts_with("底本：") {
                // 底本についての表記があったらそれ以後はメタデータなので無視する。
                break;
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
            let line = self.remove_comment(line.as_str());

            buf += (annotate(line.as_str()).with_context(|| line)? + "\n").as_str();
        }

        info!("Writing {}", ofname.to_string_lossy());
        let mut ofile = File::create(ofname)?;
        ofile.write_all(buf.as_bytes())?;

        Ok(())
    }

    fn is_kyukana(&self, src: &str) -> bool {
        self.kyukana_pattern.is_match(src)
    }

    fn remove_yomigana(&self, src: &str) -> String {
        self.yomigana_pattern.replace_all(src, "").to_string()
    }

    fn remove_comment(&self, src: &str) -> String {
        self.comment_pattern.replace_all(src, "").to_string()
    }

    fn strip_meta(&self, src: &str) -> String {
        self.sokohon_pattern
            .replace_all(
                self.meta_separator_pattern
                    .replace_all(src, "")
                    .to_string()
                    .as_str(),
                "",
            )
            .to_string()
    }

    pub fn get_file_list(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
    ) -> anyhow::Result<Vec<(String, String)>> {
        let mut result: Vec<(String, String)> = Vec::new();

        for src_file in WalkDir::new(src_dir)
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| file.metadata().unwrap().is_file())
        {
            let src_path = src_file.path();
            let dirname = src_path.parent().unwrap().file_name().unwrap();
            fs::create_dir_all(dst_dir.join(dirname))?;
            let output_file = dst_dir.join(dirname).join(src_path.file_name().unwrap());

            result.push((
                src_file.path().to_string_lossy().to_string(),
                output_file.as_path().to_string_lossy().to_string(),
            ));
        }
        Ok(result)
    }

    /// _SUCCESS ファイルを書く
    pub fn write_success_file(&self, dst_dir: &Path) -> anyhow::Result<()> {
        let mut success = File::create(dst_dir.join("_SUCCESS"))?;
        success.write_all("DONE".as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_yomigana() -> anyhow::Result<()> {
        let runner = AozoraBunkoProcessor::new()?;
        let got = runner.remove_yomigana("小《ちひ》さな");
        assert_eq!(got, "小さな");
        Ok(())
    }

    #[test]
    fn test_is_kyukana() -> anyhow::Result<()> {
        let runner = AozoraBunkoProcessor::new()?;
        assert!(!runner.is_kyukana("小さな"));
        assert!(runner.is_kyukana("ヰ"));
        Ok(())
    }

    #[test]
    fn test_strip_meta() -> anyhow::Result<()> {
        let runner = AozoraBunkoProcessor::new()?;
        assert_eq!(runner.strip_meta("fuga\nMETA\n-------------------------------------------------------\nageage\n-------------------------------------------------------\nDOOO"), "DOOO");
        assert_eq!(
            runner
                .strip_meta("META\n-------------------------------------------------------\nDOOO"),
            "DOOO"
        );
        assert_eq!(runner.strip_meta("HOGE\n底本：めためた"), "HOGE\n");
        assert_eq!(runner.strip_meta("HELLO"), "HELLO");
        assert!(!runner
            .strip_meta(
                r#"百合子

-------------------------------------------------------
【テキスト中に現れる記号について】

［＃］：入力者注　主に外字の説明や、傍点の位置の指定
（例）［＃地付き］〔一九五一年一月〕
-------------------------------------------------------

                                     "#
            )
            .contains("百合子"),);
        Ok(())
    }
}
