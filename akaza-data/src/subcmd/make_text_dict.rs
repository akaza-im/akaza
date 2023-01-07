use chrono::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

use anyhow::Result;
use encoding_rs::{EUC_JP, UTF_8};
use log::info;

use libakaza::romkan::RomKanConverter;
use libakaza::skk::ari2nasi::Ari2Nasi;
use libakaza::skk::skkdict::parse_skkdict;

/// テキスト形式での辞書を作成する。
// 070_make-system-dict.py を移植した。
pub fn make_text_dict() -> Result<()> {
    make_single_term_dict()?;
    Ok(())
}

/// 郵便番号や絵文字など、それを狙って変換したときにだけ反応して欲しいものを入れた辞書です。
/// 通常の長文を変換している時に発動するとじゃまくさいけど、なきゃないで不便なのでこういう処理にしています。
fn make_single_term_dict() -> Result<()> {
    let dictionary_sources = [
        // 先の方が優先される
        ("skk-dev-dict/SKK-JISYO.emoji", UTF_8),
        ("skk-dev-dict/zipcode/SKK-JISYO.zipcode", EUC_JP),
    ];
    let mut dicts = Vec::new();
    let ari2nasi = Ari2Nasi::new(RomKanConverter::default());
    for (path, encoding) in dictionary_sources {
        let file = File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        BufReader::new(file).read_to_end(&mut buf)?;
        let (decoded, _, _) = encoding.decode(buf.as_slice());
        let decoded = decoded.to_string();
        let (ari, nasi) = parse_skkdict(decoded.as_str())?;
        dicts.push(nasi);
        dicts.push(ari2nasi.ari2nasi(&ari)?);
    }
    dicts.push(make_lisp_dict());
    write_dict("work/jawiki.single_term.txt", dicts)?;
    Ok(())
}

fn write_dict(ofname: &str, dicts: Vec<HashMap<String, Vec<String>>>) -> anyhow::Result<()> {
    info!("Writing {}", ofname);
    let merged_dict = merge_skkdict(dicts);
    let mut wfp = File::create(ofname)?;
    for (yomi, kanjis) in merged_dict.iter() {
        let kanjis = kanjis.join("/");
        wfp.write_fmt(format_args!("{} /{}/\n", yomi, kanjis))?;
    }
    copy_snapshot(Path::new(ofname))?;
    Ok(())
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

fn copy_snapshot(path: &Path) -> Result<()> {
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
