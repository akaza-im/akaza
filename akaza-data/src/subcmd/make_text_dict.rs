use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::Result;
use chrono::prelude::*;
use encoding_rs::{EUC_JP, UTF_8};
use log::info;

use libakaza::romkan::RomKanConverter;
use libakaza::skk::ari2nasi::Ari2Nasi;

/// テキスト形式での辞書を作成する。
// 070_make-system-dict.py を移植した。
pub fn make_text_dict() -> Result<()> {
    single_term::make_single_term_dict()?;
    system_dict::make_system_dict()?;
    Ok(())
}

mod system_dict {
    use libakaza::skk::skkdict::read_skkdict;

    use super::*;

    pub fn make_system_dict() -> anyhow::Result<()> {
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
            dicts.push(nasi);
            dicts.push(ari2nasi.ari2nasi(&ari)?);
        }
        // TODO dicts.push(make_vocab_dict());
        write_dict("work/jawiki.system_dict.txt", dicts)?;
        Ok(())
    }
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
        write_dict("work/jawiki.single_term.txt", dicts)?;
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
