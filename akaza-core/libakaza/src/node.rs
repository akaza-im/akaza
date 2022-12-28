use crate::lm::system_unigram_lm::SystemUnigramLM;
use crate::user_language_model::UserLanguageModel;
use crate::UNKNOWN_WORD_ID;

pub(crate) struct Node {
    start_pos: i32,
    yomi: String,
    word: String,
    pub(crate) key: String,
    is_bos: bool,
    is_eos: bool,
    system_word_id: i32,
    system_unigram_cost: f32,
    total_cost: Option<f32>, // unigram cost + bigram cost + previous cost
    prev: Option<Box<Node>>,
    // bigram_cache: HashMap<String, f32>,
}

impl Node {
    fn new(
        start_pos: i32,
        yomi: &String,
        word: &String,
        key: &String,
        is_bos: bool,
        is_eos: bool,
        system_word_id: i32,
        system_unigram_cost: f32,
    ) -> Node {
        Node {
            start_pos,
            yomi: yomi.clone(),
            word: word.clone(),
            key: key.clone(),
            is_bos,
            is_eos,
            system_word_id,
            system_unigram_cost,
            total_cost: Option::None,
            prev: Option::None,
        }
    }

    fn new_bos_node() -> Node {
        Node {
            start_pos: -1,
            yomi: "__BOS__".to_string(),
            word: "__BOS__".to_string(),
            key: "__BOS__/__BOS__".to_string(),
            is_bos: true,
            is_eos: false,
            system_word_id: UNKNOWN_WORD_ID,
            system_unigram_cost: 0_f32,
            total_cost: None,
            prev: None,
        }
    }

    fn new_eos_node(start_pos: i32) -> Node {
        // 本来使うべきだが、key をわざと使わない。__EOS__ 考慮すると変換精度が落ちるので。。今は使わない。
        // うまく使えることが確認できれば、__EOS__/__EOS__ にする。
        Node {
            start_pos,
            yomi: "__EOS__".to_string(),
            word: "__EOS__".to_string(),
            key: "__EOS__".to_string(),
            is_bos: false,
            is_eos: true,
            system_word_id: UNKNOWN_WORD_ID,
            system_unigram_cost: 0_f32,
            total_cost: None,
            prev: None,
        }
    }

    fn create_node(
        system_unigram_lm: &SystemUnigramLM,
        start_pos: i32,
        yomi: &String,
        kanji: &String,
    ) -> Node {
        let key = kanji.clone() + "/" + yomi;
        let result = system_unigram_lm.find_unigram(&key).unwrap();
        let (word_id, cost) = result;
        Self::new(start_pos, yomi, kanji, &key, false, false, word_id, cost)
    }

    fn calc_node_cost(
        &mut self,
        user_language_model: &UserLanguageModel,
        ulm: &SystemUnigramLM,
    ) -> f32 {
        if let Some(user_cost) = user_language_model.get_unigram_cost(&self.key) {
            // use user's score, if it's exists.
            return user_cost;
        }

        if self.system_word_id != UNKNOWN_WORD_ID {
            self.total_cost = Some(self.system_unigram_cost);
            return self.system_unigram_cost;
        } else {
            // 労働者災害補償保険法 のように、システム辞書には採録されているが,
            // 言語モデルには採録されていない場合,漢字候補を先頭に持ってくる。
            return if self.word.len() < self.yomi.len() {
                // 読みのほうが短いので、漢字。
                ulm.get_default_cost_for_short()
            } else {
                ulm.get_default_cost()
            };
        }
        // calc_bigram_cost
        // get_bigram_cost
        // get_bigram_cost_from_cache
        // set_prev
        // ==
        // surface
    }
}
