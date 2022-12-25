use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use rx_sys::RXBuilder;

unsafe fn make_binary_dict(txtfile: &String, triefile: &String) {
    println!("Generating {} from {}", triefile, txtfile);

    let rx_builder = RXBuilder::new();

    let file = File::open(txtfile).expect("Open {txtfile} correctly.");
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let v: Vec<&str> = line.trim().split(" ").collect();
        let word = v[0];
        let kanjis = v[1];
        println!("word={} kanjis={}", word, kanjis);
        let key = [word.as_bytes(), b"\xff", kanjis.as_bytes(), b"\x00"].concat();
        rx_builder.add(key);
    }
    rx_builder.build();
    let image = rx_builder.get_image();
    let size = rx_builder.get_size();
    let image = std::slice::from_raw_parts(image, size as usize);

    let mut ofile = File::create(triefile).unwrap();
    ofile.write_all(image).unwrap();
}

fn main() {
    let args : Vec<String> = env::args().collect();
    let txtfile = &args[1];
    let triefile =&args[2];
    unsafe { make_binary_dict(txtfile, triefile); }
}

