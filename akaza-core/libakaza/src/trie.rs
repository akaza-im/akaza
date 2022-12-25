use std::fs::File;
use std::io::Write;
use rx_sys::RXBuilder;

// RX を使っているか、MARISA をつかっているか、などの実装詳細は
// このファイルで隠蔽される。

pub struct TrieBuilder {
    rx_builder: RXBuilder,
}
impl TrieBuilder {
    pub unsafe fn new() -> TrieBuilder {
        TrieBuilder { rx_builder: RXBuilder::new() }
    }

    pub unsafe fn add(&self, key: Vec<u8>) {
        self.rx_builder.add(key);
    }

    pub unsafe fn save(&self, ofname: &String) -> std::io::Result<()> {
        self.rx_builder.build();
        let image = self.rx_builder.get_image();
        let size = self.rx_builder.get_size();
        let image = std::slice::from_raw_parts(image, size as usize);

        let mut ofile = File::create(ofname).unwrap();
        return ofile.write_all(image);
    }
}

#[test]
fn test() {
    unsafe {
        let builder = TrieBuilder::new();
        builder.add("foobar".as_bytes());
        builder.save("/tmp/dump.trie".to_string()).unwrap();

    }
}