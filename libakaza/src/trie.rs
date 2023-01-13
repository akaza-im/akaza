use anyhow::Result;

use marisa_sys::{Keyset, Marisa};

// Trie 木の実装詳細はこのファイルで隠蔽される。

#[derive(Default)]
pub struct TrieBuilder {
    keyset: Keyset,
}

impl TrieBuilder {
    pub fn add(&mut self, key: Vec<u8>) {
        self.keyset.push_back(key.as_slice());
    }

    pub fn save(&self, ofname: &str) -> Result<()> {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        marisa.save(ofname)?;
        Ok(())
    }

    pub fn build(&self) -> Trie {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        Trie { marisa }
    }
}

// Load trie from file.
// predictive search
pub struct Trie {
    pub marisa: Marisa,
}

impl Trie {
    pub fn load(filename: &str) -> Result<Trie> {
        let mut marisa = Marisa::default();
        marisa.load(filename)?;
        Ok(Trie { marisa })
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
        p
    }

    pub fn num_keys(&self) -> usize {
        self.marisa.num_keys()
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
        let mut builder = TrieBuilder::default();
        builder.add("foobar".as_bytes().to_vec());
        builder.save("/tmp/dump.trie").unwrap();

        let trie = Trie::load("/tmp/dump.trie").unwrap();
        let result = trie.predictive_search("foobar".to_string().into_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(
            String::from_utf8((result[0].keyword).clone()).unwrap(),
            "foobar"
        );
        assert_eq!(result[0].id, 0);
    }
}
