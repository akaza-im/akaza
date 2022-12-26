use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::Write;

use rx_sys::{Rx, RXBuilder};

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

// Load trie from file.
// predictive search
pub struct Trie {
    rx: Rx,
}

impl Trie {
    pub unsafe fn load(filename: &String) -> Result<Trie, Error> {
        let content = fs::read(filename);
        return match content {
            Ok(mut content) => {
                let ptr = content.as_mut_ptr();
                Ok(Trie { rx: Rx::open(ptr) })
            }
            Err(error) => {
                Err(error)
            }
        };
    }

    pub unsafe fn predictive_search(&self, keyword: Vec<u8>) -> Vec<SearchResult> {
        let mut p: Vec<SearchResult> = Vec::new();
        self.rx.search(1, keyword, |keyword, len, id| {
            p.push(SearchResult { keyword, len, id });
            1
        });
        return p;
    }
}

pub struct SearchResult {
    pub keyword: String,
    pub len: i32,
    pub id: i32,
}

#[test]
fn test() {
    unsafe {
        let builder = TrieBuilder::new();
        builder.add("foobar\0".as_bytes().to_vec());
        builder.save(&"/tmp/dump.trie".to_string()).unwrap();

        let trie = Trie::load(&"/tmp/dump.trie".to_string()).unwrap();
        let result = trie.predictive_search("foobar\0".to_string().into_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].keyword, "foobar");
        assert_eq!(result[0].id, 0);
        assert_eq!(result[0].len, 6);
    }
}
