use anyhow::Context;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use kelp::{kata2hira, ConvOption};
use log::info;
use regex::Regex;
use vibrato::{Dictionary, Tokenizer};
use walkdir::WalkDir;

pub struct VibtaroRunner {
    tokenizer: Tokenizer,
    pub alnum_pattern: Regex,
}

impl VibtaroRunner {
    pub fn new() -> anyhow::Result<VibtaroRunner> {
        let dict = Dictionary::read(File::open("work/mecab/ipadic-mecab-2_7_0/system.dic")?)?;
        let dict = dict
            .reset_user_lexicon_from_reader(Some(File::open(
                "jawiki-kana-kanji-dict/mecab-userdic.csv",
            )?))
            .with_context(|| "Opening userdic")?;

        let tokenizer = vibrato::Tokenizer::new(dict);

        // 英数/記号のみの行を無視するための正規表現。
        // 75||19||colspan=2|-||1||0||76||19
        let alnum_pattern = Regex::new("^[a-zA-Z0-9|=-]+")?;

        Ok(VibtaroRunner {
            tokenizer,
            alnum_pattern,
        })
    }

    pub fn process_file(&self, ifname: &Path, ofname: &Path) -> anyhow::Result<()> {
        let file = File::open(ifname)?;
        let mut buf = String::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            let line = line.trim();
            if line.starts_with("<") {
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

            buf += self.annotate(line)?.as_str();
        }
        let mut ofile = File::create(ofname)?;
        ofile.write_all(buf.as_bytes())?;
        Ok(())
    }

    /// Vibrato を利用してファイルをアノテーションします。
    pub fn annotate(&self, src: &str) -> anyhow::Result<String> {
        let mut worker = self.tokenizer.new_worker();

        worker.reset_sentence(src);
        worker.tokenize();

        let mut buf = String::new();

        // TODO 連結処理的なことが必要ならする。
        // Vibrato/mecab の場合、接尾辞などが細かく分かれることは少ないが、
        // 一方で、助詞/助動詞などが細かくわかれがち。
        for i in 0..worker.num_tokens() {
            let token = worker.token(i);
            let feature: Vec<&str> = token.feature().split(',').collect();
            // if feature.len() <= 7 {
            //     println!("Too few features: {}/{}", token.surface(), token.feature())
            // }

            // let hinshi = feature[0];
            let yomi = if feature.len() > 7 {
                feature[7]
            } else {
                // 読みがな不明なもの。固有名詞など。
                // サンデフィヨルド・フォトバル/名詞,固有名詞,組織,*,*,*,*
                token.surface()
            };
            let yomi = kata2hira(yomi, ConvOption::default());
            buf += format!("{}/{} ", token.surface(), yomi).as_str();
            // println!("{}/{}/{}", token.surface(), hinshi, yomi);
        }

        Ok(buf.trim().to_string() + "\n")
    }
}

pub fn annotate_wikipedia() -> anyhow::Result<()> {
    let output_dir = Path::new("work/mecab/wikipedia-annotated/");
    let runner = VibtaroRunner::new()?;

    // TODO parallel processing
    for src_file in WalkDir::new("work/extracted")
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
    {
        let src_path = src_file.path();
        let dirname = src_path.parent().unwrap().file_name().unwrap();
        fs::create_dir_all(output_dir.join(dirname))?;
        let output_file = output_dir.join(dirname).join(src_path.file_name().unwrap());
        info!(
            "{} => {}",
            src_file.path().display(),
            output_file.as_path().display()
        );
        runner.process_file(src_file.path(), &output_file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let runner = VibtaroRunner::new()?;
        runner.annotate("私の名前は中野です。")?;
        Ok(())
    }

    #[test]
    fn test_wikipedia() -> anyhow::Result<()> {
        let runner = VibtaroRunner::new()?;

        let fname = "work/extracted/BE/wiki_02";
        fs::create_dir_all("work/mecab/wikipedia-annotated/BE/")?;
        runner.process_file(
            Path::new(fname),
            Path::new("work/mecab/wikipedia-annotated/BE/wiki_02"),
        )?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_all() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        annotate_wikipedia()?;
        Ok(())
    }
}
