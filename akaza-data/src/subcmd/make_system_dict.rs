use std::fs::File;
use std::io::{prelude::*, BufReader};

use anyhow::Result;
use log::trace;

use libakaza::kana_kanji_dict::KanaKanjiDictBuilder;

pub fn make_system_dict(txtfile: &String, triefile: &String) -> Result<()> {
    println!("Generating {} from {}", triefile, txtfile);

    let mut kana_kanji_dict = KanaKanjiDictBuilder::default();

    let file = File::open(txtfile).expect("Open {txtfile} correctly.");
    for line in BufReader::new(file).lines() {
        let line = line?;
        let v: Vec<&str> = line.trim().split(' ').collect();
        if v.len() != 2 {
            continue;
        }
        let yomi = v[0];
        let kanjis = v[1];
        trace!("word={} kanjis={}", yomi, kanjis);
        kana_kanji_dict.add(yomi, kanjis);
    }
    kana_kanji_dict.save(triefile)?;
    Ok(())
}
