use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::Context;
use regex::Regex;

use crate::corpus_reader::base::CorpusReader;

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
        let yomigana_pattern = Regex::new(r"[（\(][\u3041-\u309F、]+[）)]")?;

        Ok(ExtractedWikipediaProcessor {
            alnum_pattern,
            yomigana_pattern,
        })
    }

    fn remove_yomigana<'a>(&self, src: &'a str) -> Cow<'a, str> {
        self.yomigana_pattern.replace_all(src, "")
    }
}

impl CorpusReader for ExtractedWikipediaProcessor {
    fn process_file<F>(&self, ifname: &Path, ofname: &Path, annotate: &mut F) -> anyhow::Result<()>
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
            // &lt;section begin="WEST順位" /&gt;
            //  class="wikitable sortable" style="text-align: center;"
            // ! !!class="unsortable"| チーム!!!!!!!!!!!!!!!!!!!!出場権または降格
            // &lt;section end="WEST順位" /&gt;
            // !colspan="4"|日本!!colspan="2"|リーグ戦!!colspan="2"|!!colspan="2"|天皇杯!!colspan="2"|期間通算
            if line.starts_with(" class=") || line.starts_with("&lt;") || line.starts_with('!') {
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

            buf.push_str(&annotate(&line).with_context(|| line.into_owned())?);
            buf.push('\n');
        }
        let mut ofile = File::create(ofname)?;
        ofile.write_all(buf.as_bytes())?;
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
