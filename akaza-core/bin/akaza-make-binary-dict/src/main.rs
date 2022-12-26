use std::env;
use std::fs::File;
use std::io::{BufReader, prelude::*};

use libakaza::trie::TrieBuilder;

unsafe fn make_binary_dict(txtfile: &String, triefile: &String) {
    println!("Generating {} from {}", triefile, txtfile);

    let trie_builder = TrieBuilder::new();

    let file = File::open(txtfile).expect("Open {txtfile} correctly.");
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let v: Vec<&str> = line.trim().split(" ").collect();
        if v.len() != 2 {
            break;
        }
        let word = v[0];
        let kanjis = v[1];
        println!("word={} kanjis={}", word, kanjis);
        let key = [word.as_bytes(), b"\xff", kanjis.as_bytes(), b"\x00"].concat();
        trie_builder.add(key);
    }
    trie_builder.save(triefile).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let txtfile = &args[1];
    let triefile = &args[2];
    unsafe { make_binary_dict(txtfile, triefile); }
}

