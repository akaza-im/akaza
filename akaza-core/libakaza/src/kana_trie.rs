use marisa_sys::{Keyset, Marisa};

pub(crate) struct KanaTrieBuilder {
    keyset: Keyset,
}

impl KanaTrieBuilder {
    pub(crate) fn new() -> KanaTrieBuilder {
        KanaTrieBuilder {
            keyset: Keyset::new(),
        }
    }

    pub(crate) fn add(&mut self, yomi: &String) {
        self.keyset.push_back(yomi.as_bytes());
    }

    pub(crate) fn build(&self) -> KanaTrie {
        let mut marisa = Marisa::new();
        marisa.build(&self.keyset);
        KanaTrie::new(marisa)
    }
}

pub(crate) struct KanaTrie {
    marisa: Marisa,
}

impl KanaTrie {
    pub(crate) fn new(marisa: Marisa) -> KanaTrie {
        KanaTrie { marisa }
    }

    pub(crate) fn save(&self, file_name: &String) -> Result<(), String> {
        self.marisa.save(file_name)
    }

    pub(crate) fn load(file_name: &String) -> Result<KanaTrie, String> {
        let mut marisa = Marisa::new();
        match marisa.load(file_name) {
            Ok(_) => Ok(KanaTrie { marisa }),
            Err(err) => Err(err),
        }
    }

    pub(crate) fn common_prefix_search(&self, query: &String) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        self.marisa.common_prefix_search(query, |word, _| {
            result.push(String::from_utf8(word.to_vec()).unwrap());
            true
        });
        return result;
    }
}
