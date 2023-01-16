use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::{bail, Result};
use encoding_rs::UTF_8;
use log::info;

use crate::utils::copy_snapshot;

/// テキスト形式での辞書を作成する。
pub fn make_system_dict(
    txt_file: &str,
    vocab_file_path: Option<&str>,
    corpus_files: Vec<String>,
) -> Result<()> {
    system_dict::make_system_dict(txt_file, vocab_file_path, corpus_files)?;
    Ok(())
}

mod system_dict {
    use std::io::BufReader;

    use anyhow::{bail, Context};

    use libakaza::corpus::read_corpus_file;
    use libakaza::dict::skk::read::read_skkdict;
    use libakaza::dict::skk::write::write_skk_dict;

    use super::*;

    pub fn make_system_dict(
        txt_file: &str,
        vocab_file_path: Option<&str>,
        corpus_files: Vec<String>,
    ) -> Result<()> {
        // vocab, corpus, dict/SKK-JISYO.akaza から辞書を生成する
        let mut dicts = Vec::new();

        // SKK-JISYO.akaza を読む
        dicts.push(
            validate_dict(cleanup_dict(&read_skkdict(
                Path::new("dict/SKK-JISYO.akaza"),
                UTF_8,
            )?))
            .with_context(|| "dict/SKK-JISYO.akaza".to_string())?,
        );
        // vocab ファイルを読む
        if let Some(vocab_file_path) = vocab_file_path {
            info!("Using vocab file: {}", vocab_file_path);
            dicts.push(
                validate_dict(make_vocab_dict(vocab_file_path)?)
                    .with_context(|| "make_vocab_dict".to_string())?,
            );
        }
        // コーパスからも語彙を追加する
        dicts.push(
            validate_dict(make_corpus_dict(corpus_files)?)
                .with_context(|| "make_corpus_dict".to_string())?,
        );
        write_skk_dict(txt_file, dicts)?;
        copy_snapshot(Path::new(txt_file))?;
        Ok(())
    }

    fn cleanup_dict(dict: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
        // 全角空白が入っているとテキスト処理時におかしくなりがちなので調整。
        dict.iter()
            .map(|(k, vs)| {
                (
                    k.to_string(),
                    vs.iter()
                        .filter(|m| m.as_str() != "\u{3000}")
                        .map(|s| s.to_string())
                        .collect(),
                )
            })
            .collect::<HashMap<String, Vec<String>>>()
    }

    fn make_corpus_dict(corpus_files: Vec<String>) -> Result<HashMap<String, Vec<String>>> {
        let mut words: Vec<(String, String)> = Vec::new();

        for corpus_file in corpus_files {
            let corpus_vec = read_corpus_file(Path::new(corpus_file.as_str()))?;
            for corpus in corpus_vec {
                for node in corpus.nodes {
                    // info!("Add {}/{}", node.yomi, node.kanji);
                    words.push((node.yomi.to_string(), node.surface.to_string()));
                }
            }
        }

        Ok(grouping_words(words))
    }

    fn grouping_words(words: Vec<(String, String)>) -> HashMap<String, Vec<String>> {
        words.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<String, Vec<String>>, t: &(String, String)| {
                let (p, q) = t;
                acc.entry(p.to_string())
                    .or_insert_with(Vec::new)
                    .push(q.to_string());
                acc
            },
        )
    }

    fn make_vocab_dict(vocab_file_path: &str) -> Result<HashMap<String, Vec<String>>> {
        let rfp = File::open(vocab_file_path)?;
        let mut words: Vec<(String, String)> = Vec::new();
        for line in BufReader::new(rfp).lines() {
            let line = line?;
            let Some((surface, yomi)) = line.split_once('/') else {
                bail!("Cannot parse vocab file: {:?} in {}", line, vocab_file_path);
            };
            if yomi == "UNK" {
                // なんのときに発生するかはわからないが、なにか意味がありそうな処理。
                // Python 版にあったので残してある。たぶんいらない処理。
                continue;
            }
            if yomi.contains('\u{3000}') || surface.contains('\u{3000}') {
                // 全角空白はいってるのはおかしい
                continue;
            }
            if yomi.is_empty() {
                // よみがないのはおかしい。
                continue;
            }
            words.push((yomi.to_string(), surface.to_string()));
        }
        Ok(grouping_words(words))
    }
}

fn validate_dict(dict: HashMap<String, Vec<String>>) -> Result<HashMap<String, Vec<String>>> {
    for (kana, surfaces) in dict.iter() {
        if kana.is_empty() {
            bail!("Kana must not be empty: {:?}", surfaces);
        }
        let kana_cnt = kana.chars().count();
        for surface in surfaces {
            if surface.is_empty() {
                bail!("Empty surface: {:?}", kana);
            }
            if kana_cnt == 1 && kana_cnt < surface.chars().count() {
                // info!("Missing surface: {}<{}", kana, surface);
            }
            if kana == "い" && kana_cnt < surface.chars().count() {
                bail!("XXX Missing surface: {:?}<{:?}", kana, surface);
            }
            if kana == "い" && surface == "好い" {
                bail!("Missing surface: {}<{}", kana, surface);
            }
            if kana.contains('\u{3000}') {
                bail!("Full width space: {}<{}", kana, surface);
            }
        }
    }
    Ok(dict)
}
