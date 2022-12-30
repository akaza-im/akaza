use marisa_sys::{Keyset, Marisa};

pub struct KanaTrieBuilder {
    keyset: Keyset,
}

impl KanaTrieBuilder {
    pub fn new() -> KanaTrieBuilder {
        KanaTrieBuilder {
            keyset: Keyset::default(),
        }
    }

    pub fn add(&mut self, yomi: &String) {
        self.keyset.push_back(yomi.as_bytes());
    }

    pub fn build(&self) -> KanaTrie {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset);
        KanaTrie::new(marisa)
    }
}

pub struct KanaTrie {
    marisa: Marisa,
}

impl KanaTrie {
    pub(crate) fn new(marisa: Marisa) -> KanaTrie {
        KanaTrie { marisa }
    }

    pub(crate) fn save(&self, file_name: &str) -> Result<(), String> {
        self.marisa.save(file_name)
    }

    pub fn load(file_name: &str) -> Result<KanaTrie, String> {
        let mut marisa = Marisa::default();
        marisa.load(file_name)?;
        Ok(KanaTrie { marisa })
    }

    pub(crate) fn common_prefix_search(&self, query: &String) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        self.marisa.common_prefix_search(query, |word, _| {
            result.push(String::from_utf8(word.to_vec()).unwrap());
            true
        });
        result
    }
}
