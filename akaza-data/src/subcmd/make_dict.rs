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
    unidic_file: String,
    sudachi_lex_files: Vec<String>,
) -> Result<()> {
    system_dict::make_system_dict(
        txt_file,
        vocab_file_path,
        corpus_files,
        unidic_file,
        sudachi_lex_files,
    )?;
    Ok(())
}

mod system_dict {
    use std::io::BufReader;

    use anyhow::{bail, Context};
    use log::trace;
    use regex::Regex;

    use libakaza::corpus::read_corpus_file;
    use libakaza::dict::skk::read::read_skkdict;
    use libakaza::dict::skk::write::write_skk_dict_with_header;

    use super::*;

    pub fn make_system_dict(
        txt_file: &str,
        vocab_file_path: Option<&str>,
        corpus_files: Vec<String>,
        unidic_file: String,
        sudachi_lex_files: Vec<String>,
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
        // unidic からも語彙を追加する
        dicts.push(
            validate_dict(make_unidic_dict(unidic_file)?)
                .with_context(|| "make_unidic_dict".to_string())?,
        );
        // Sudachi 辞書から固有名詞を追加する（最低優先度）
        if !sudachi_lex_files.is_empty() {
            dicts.push(
                validate_dict(make_sudachi_dict(&sudachi_lex_files)?)
                    .with_context(|| "make_sudachi_dict".to_string())?,
            );
        }
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let version = env!("CARGO_PKG_VERSION");
        let mut header_comments = vec![
            format!("AKAZA_DATA_VERSION: {version}"),
            format!("BUILD_TIMESTAMP: {now}"),
        ];
        if !sudachi_lex_files.is_empty() {
            header_comments.push(
                "Contains proper noun data from Sudachi dictionary (Apache-2.0, WAQ LLC)"
                    .to_string(),
            );
        }
        // 読みが通常のひらがな (ぁ-ゔ) で始まらないエントリはかな漢字変換辞書として無意味なので除外
        let dicts = dicts
            .into_iter()
            .map(|dict| {
                dict.into_iter()
                    .filter(|(yomi, _)| yomi.starts_with(|c: char| is_hiragana(c)))
                    .collect()
            })
            .collect();
        write_skk_dict_with_header(txt_file, dicts, &header_comments)?;
        copy_snapshot(Path::new(txt_file))?;
        post_validate(txt_file)?;
        Ok(())
    }

    /// 出来上がった辞書が問題ない品質かを確認する
    fn post_validate(path: &str) -> Result<()> {
        let dict = read_skkdict(Path::new(path), UTF_8)?;
        for key in ["あぐりげーしょん"] {
            if !dict.contains_key(key) {
                bail!("Missing key in dict: {}", key);
            }
        }
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
                acc.entry(p.to_string()).or_default().push(q.to_string());
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

    // あぐりげーしょん、などのカタカナ語を unidic から拾う。
    fn make_unidic_dict(path: String) -> anyhow::Result<HashMap<String, Vec<String>>> {
        let file = File::open(path)?;
        let mut dict = HashMap::new();
        let katakana_pattern = Regex::new(r"^\p{wb=Katakana}+")?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            let csv = line.split(',').collect::<Vec<_>>();
            if csv.len() < 10 {
                trace!("Incomplete line: {:?}", line);
                continue;
            }

            // コストは低い方がよく出てくるもの。
            let surface = csv[0];
            let _cost = csv[2];
            let _hinshi = csv[4];
            let _subhinshi = csv[5];
            let yomi = csv[10];

            if katakana_pattern.is_match(surface)
                && katakana_pattern.is_match(yomi)
                && surface == yomi
            {
                dict.insert(
                    crate::tokenizer::base::kata2hira_string(surface),
                    vec![yomi.to_string()],
                );
            }
        }
        info!("Got {} entries from unidic", dict.len());
        Ok(dict)
    }

    /// Sudachi 辞書 CSV から固有名詞（名詞-固有名詞-一般）を読み込む。
    /// 人名・地名は同音異義語の衝突リスクがあるため除外する。
    fn make_sudachi_dict(paths: &[String]) -> Result<HashMap<String, Vec<String>>> {
        let mut words: Vec<(String, String)> = Vec::new();

        for path in paths {
            let file = File::open(path)
                .with_context(|| format!("Failed to open Sudachi lex file: {}", path))?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                let csv: Vec<&str> = line.split(',').collect();
                if csv.len() < 13 {
                    trace!("Sudachi: incomplete line: {:?}", line);
                    continue;
                }

                let pos1 = csv[5]; // 品詞大分類
                let pos2 = csv[6]; // 品詞中分類
                let pos3 = csv[7]; // 品詞小分類

                // 名詞-固有名詞-一般 のみ取り込む
                if pos1 != "名詞" || pos2 != "固有名詞" || pos3 != "一般" {
                    continue;
                }

                let surface = csv[12]; // 表記形（表示用表層形）
                let yomi_kata = csv[11]; // 読み（カタカナ）

                if surface.is_empty() || yomi_kata.is_empty() {
                    continue;
                }

                // 全角空白を含むエントリはスキップ
                if surface.contains('\u{3000}') || yomi_kata.contains('\u{3000}') {
                    continue;
                }

                // 表層形に日本語文字（ひらがな・カタカナ・漢字）を含まないものはスキップ
                if !surface.chars().any(|c| {
                    is_hiragana(c)
                        || ('\u{30A0}'..='\u{30FF}').contains(&c)
                        || ('\u{4E00}'..='\u{9FFF}').contains(&c)
                        || ('\u{3400}'..='\u{4DBF}').contains(&c)
                }) {
                    continue;
                }

                let yomi = crate::tokenizer::base::kata2hira_string(yomi_kata);
                if yomi.is_empty() {
                    continue;
                }

                words.push((yomi, surface.to_string()));
            }
        }

        let dict = grouping_words(words);
        info!("Got {} entries from Sudachi dictionaries", dict.len());
        Ok(dict)
    }

    /// 通常のひらがな文字 (ぁ〜ゔ) かどうか判定する。
    /// 濁点・半濁点記号 (U+3099-U+309C) 等の特殊文字は除外。
    fn is_hiragana(c: char) -> bool {
        ('\u{3041}'..='\u{3094}').contains(&c)
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
