use marisa_sys::{Keyset, Marisa};

// Trie 木の実装詳細はこのファイルで隠蔽される。

#[derive(Default)]
pub struct TrieBuilder {
    pub keyset: Keyset,
}

// Load trie from file.
// predictive search
pub struct Trie {
    pub marisa: Marisa,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub keyword: Vec<u8>,
    pub id: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        {
            let mut builder = TrieBuilder::default();
            let key = "foobar".as_bytes().to_vec();
            builder.keyset.push_back(key.as_slice());
            let mut marisa = Marisa::default();
            marisa.build(&builder.keyset);
            marisa.save("/tmp/dump.trie")?;

            let mut marisa = Marisa::default();
            marisa.load("/tmp/dump.trie")?;
            let keyword = "foobar".to_string().into_bytes();
            let mut result: Vec<SearchResult> = Vec::new();
            marisa.predictive_search(keyword.as_slice(), |key, id| {
                result.push(SearchResult {
                    keyword: key.to_vec(),
                    id,
                });
                true
            });
            assert_eq!(result.len(), 1);
            assert_eq!(
                String::from_utf8((result[0].keyword).clone()).unwrap(),
                "foobar"
            );
            assert_eq!(result[0].id, 0);
        }
        Ok(())
    }
}
