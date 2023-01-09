use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::{bail, Result};
use encoding_rs::{EUC_JP, UTF_8};
use log::info;

use libakaza::romkan::RomKanConverter;
use libakaza::skk::ari2nasi::Ari2Nasi;

use crate::utils::copy_snapshot;

/// テキスト形式での辞書を作成する。
// 070_make-system-dict.py を移植した。
pub fn make_text_dict(vocab_file_path: Option<&str>) -> Result<()> {
    single_term::make_single_term_dict()?;
    system_dict::make_system_dict(vocab_file_path)?;
    Ok(())
}

mod system_dict {
    use std::io::BufReader;

    use anyhow::{bail, Context};

    use libakaza::corpus::read_corpus_file;
    use libakaza::skk::skkdict::read_skkdict;

    use super::*;

    pub fn make_system_dict(vocab_file_path: Option<&str>) -> anyhow::Result<()> {
        let dictionary_sources = [
            // 先の方が優先される
            ("skk-dev-dict/SKK-JISYO.L", EUC_JP),
            ("skk-dev-dict/SKK-JISYO.jinmei", EUC_JP),
            ("skk-dev-dict/SKK-JISYO.station", EUC_JP),
            ("jawiki-kana-kanji-dict/SKK-JISYO.jawiki", UTF_8),
            ("dict/SKK-JISYO.akaza", UTF_8),
        ];
        let mut dicts = Vec::new();
        let ari2nasi = Ari2Nasi::new(RomKanConverter::default());

        for (path, encoding) in dictionary_sources {
            let (ari, nasi) = read_skkdict(Path::new(path), encoding)?;
            dicts.push(validate_dict(nasi).with_context(|| path.to_string())?);
            dicts.push(validate_dict(ari2nasi.ari2nasi(&ari)?).with_context(|| path.to_string())?);
        }
        if let Some(vocab_file_path) = vocab_file_path {
            info!("Using vocab file: {}", vocab_file_path);
            dicts.push(
                validate_dict(make_vocab_dict(vocab_file_path)?)
                    .with_context(|| "make_vocab_dict".to_string())?,
            );
        }
        dicts.push(
            validate_dict(make_corpus_dict()?).with_context(|| "make_corpus_dict".to_string())?,
        );
        write_dict("work/stats-kytea/jawiki.system_dict.txt", dicts)?;
        Ok(())
    }

    fn make_corpus_dict() -> Result<HashMap<String, Vec<String>>> {
        let mut words: Vec<(String, String)> = Vec::new();

        let corpus_vec = read_corpus_file(Path::new("corpus/must.txt"))?;
        for corpus in corpus_vec {
            for node in corpus.nodes {
                // info!("Add {}/{}", node.yomi, node.kanji);
                words.push((node.yomi.to_string(), node.kanji.to_string()));
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
                bail!("Cannot parse vocab file: {}", line);
            };
            if yomi == "UNK" {
                // なんのときに発生するかはわからないが、なにか意味がありそうな処理。
                // Python 版にあったので残してある。たぶんいらない処理。
                continue;
            }
            words.push((yomi.to_string(), surface.to_string()));
        }
        Ok(grouping_words(words))
    }

    /*
    def scan_vocab():
        with open('work/stats-kytea/jawiki.vocab', 'r') as rfp:
            for line in rfp:
                word = line.rstrip()
                m = word.split('/')
                if len(m) != 2:
                    continue

                word, kana = m
                if kana == 'UNK':
                    continue
                yield word, kana


    def make_vocab_dict():
        okuri_nasi = {}

        for word, kana in scan_vocab():
            if kana not in okuri_nasi:
                okuri_nasi[kana] = []
            okuri_nasi[kana].append(word)

        return okuri_nasi
         */
}

mod single_term {
    use libakaza::skk::skkdict::read_skkdict;

    use super::*;

    /// 郵便番号や絵文字など、それを狙って変換したときにだけ反応して欲しいものを入れた辞書です。
    /// 通常の長文を変換している時に発動するとじゃまくさいけど、なきゃないで不便なのでこういう処理にしています。
    pub(crate) fn make_single_term_dict() -> Result<()> {
        let dictionary_sources = [
            // 先の方が優先される
            ("skk-dev-dict/SKK-JISYO.emoji", UTF_8),
            ("skk-dev-dict/zipcode/SKK-JISYO.zipcode", EUC_JP),
        ];
        let mut dicts = Vec::new();
        let ari2nasi = Ari2Nasi::new(RomKanConverter::default());
        for (path, encoding) in dictionary_sources {
            let (ari, nasi) = read_skkdict(Path::new(path), encoding)?;
            dicts.push(nasi);
            dicts.push(ari2nasi.ari2nasi(&ari)?);
        }
        dicts.push(make_lisp_dict());
        write_dict("work/stats-kytea/jawiki.single_term.txt", dicts)?;
        Ok(())
    }

    fn make_lisp_dict() -> HashMap<String, Vec<String>> {
        HashMap::from([(
            "きょう".to_string(),
            vec![
                "(strftime (current-datetime) \"%Y-%m-%d\")".to_string(),
                "(strftime (current-datetime) \"%Y年%m月%d日\")".to_string(),
                "(strftime (current-datetime) \"%Y年%m月%d日(%a)\")".to_string(),
            ],
        )])
    }
}

fn write_dict(ofname: &str, dicts: Vec<HashMap<String, Vec<String>>>) -> anyhow::Result<()> {
    info!("Writing {}", ofname);
    let merged_dict = merge_skkdict(dicts);
    let mut wfp = File::create(ofname)?;
    for (yomi, kanjis) in merged_dict.iter() {
        let kanjis = kanjis.join("/");
        // この出力先ファイルは、「SKK 風」だが、SKK 辞書ではない。
        // 前後に slash は入れなくて良い。
        wfp.write_fmt(format_args!("{} {}\n", yomi, kanjis))?;
    }
    copy_snapshot(Path::new(ofname))?;
    Ok(())
}

fn validate_dict(dict: HashMap<String, Vec<String>>) -> Result<HashMap<String, Vec<String>>> {
    for (kana, surfaces) in dict.iter() {
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
        }
    }
    Ok(dict)
}

fn merge_skkdict(dicts: Vec<HashMap<String, Vec<String>>>) -> BTreeMap<String, Vec<String>> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // 取りうるキーをリストアップする
    let mut keys: HashSet<String> = HashSet::new();
    for dic in &dicts {
        for key in dic.keys() {
            keys.insert(key.to_string());
        }
    }

    // それぞれのキーについて、候補をリストアップする
    for key in keys {
        let mut kanjis: Vec<String> = Vec::new();

        for dic in &dicts {
            if let Some(kkk) = dic.get(&key.to_string()) {
                for k in kkk {
                    if !kanjis.contains(k) {
                        kanjis.push(k.clone());
                    }
                }
            }
        }

        result.insert(key, kanjis);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// コーパスファイルをちゃんと処理できているか
    #[test]
    fn test_corpus() -> Result<()> {
        // 処理する
        make_text_dict(None)?;

        let mut file = File::open("work/stats-kytea/jawiki.system_dict.txt")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        assert!(buf.contains("ぱーせぷとろん パーセプトロン"));

        Ok(())
    }
}
