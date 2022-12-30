use std::io::Error;

use marisa_sys::{Keyset, Marisa};

// Trie 木の実装詳細はこのファイルで隠蔽される。

pub struct TrieBuilder {
    keyset: Keyset,
}

impl TrieBuilder {
    pub fn new() -> TrieBuilder {
        TrieBuilder {
            keyset: Keyset::new(),
        }
    }

    pub fn add(&mut self, key: Vec<u8>) {
        self.keyset.push_back(key.as_slice());
    }

    pub fn save(&self, ofname: &String) -> std::io::Result<()> {
        let mut marisa = Marisa::new();
        marisa.build(&self.keyset);
        marisa.save(ofname).unwrap();
        return Ok(());
    }

    pub fn build(&self) -> Trie {
        let mut marisa = Marisa::new();
        marisa.build(&self.keyset);
        Trie { marisa }
    }
}

// Load trie from file.
// predictive search
pub struct Trie {
    marisa: Marisa,
}

impl Trie {
    pub fn load(filename: &String) -> Result<Trie, Error> {
        let mut marisa = Marisa::new();
        marisa.load(filename).unwrap();
        return Ok(Trie { marisa });
    }

    pub fn predictive_search(&self, keyword: Vec<u8>) -> Vec<SearchResult> {
        let mut p: Vec<SearchResult> = Vec::new();
        self.marisa
            .predictive_search(keyword.as_slice(), |key, id| {
                p.push(SearchResult {
                    keyword: key.to_vec(),
                    id,
                });
                true
            });
        return p;
    }

    pub fn num_keys(&self) -> usize {
        return self.marisa.num_keys();
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub keyword: Vec<u8>,
    pub id: usize,
}

#[test]
fn test() {
    {
        let mut builder = TrieBuilder::new();
        builder.add("foobar".as_bytes().to_vec());
        builder.save(&"/tmp/dump.trie".to_string()).unwrap();

        let trie = Trie::load(&"/tmp/dump.trie".to_string()).unwrap();
        let result = trie.predictive_search("foobar".to_string().into_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(
            String::from_utf8((result[0].keyword).clone()).unwrap(),
            "foobar"
        );
        assert_eq!(result[0].id, 0);
    }
}
