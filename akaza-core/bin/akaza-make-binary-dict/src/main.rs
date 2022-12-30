use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use libakaza::kana_kanji_dict::KanaKanjiDictBuilder;

unsafe fn make_binary_dict(txtfile: &String, triefile: &String) {
    println!("Generating {} from {}", triefile, txtfile);

    let mut binary_dict = KanaKanjiDictBuilder::new();

    let file = File::open(txtfile).expect("Open {txtfile} correctly.");
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let v: Vec<&str> = line.trim().split(" ").collect();
        if v.len() != 2 {
            continue;
        }
        let yomi = v[0];
        let kanjis = v[1];
        println!("word={} kanjis={}", yomi, kanjis);
        binary_dict.add(&yomi.to_string(), &kanjis.to_string());
    }
    binary_dict.save(triefile).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let txtfile = &args[1];
    let triefile = &args[2];
    unsafe {
        make_binary_dict(txtfile, triefile);
    }
}
