#include "../include/akaza.h"

akaza::Node akaza::Node::create_bos() {
    return akaza::Node(0, "__BOS__", "__BOS__");
}

akaza::Node akaza::Node::create_eos() {
    return akaza::Node(0, "__EOS__", "__EOS__");
}

/*
    def calc_node_cost(self, user_language_model, system_language_model) -> float:
        key = self.get_key()
        u = user_language_model.get_unigram_cost(key)
        if u is not None:
            # self.logger.info(f"Use user score: {node.get_key()} -> {u}")
            return u
        # print(f"SYSTEM LANGUAGE MODEL UNIGRAM: {key}")
        word_id, score = system_language_model.find_unigram(key)
        self.id = word_id
        return score if word_id >= 0 else UNIGRAM_DEFAULT_COST
 */
float akaza::Node::calc_node_cost(
        const akaza::UserLanguageModel &user_language_model,
        const akaza::SystemUnigramLM &ulm
) {
    auto key = this->get_key();
    auto u = user_language_model.get_unigram_cost(key);
    if (u.has_value()) {
        return *u;
    }
    auto[word_id, score] = ulm.find_unigram(key);
    this->word_id = word_id;
    if (word_id != akaza::UNKNOWN_WORD_ID) {
        return score;
    } else {
        return ulm.get_default_cost();
    }
}


/*
    @staticmethod
    def _calc_bigram_cost(prev_node, next_node, user_language_model, system_language_model) -> float:
        # self → node で処理する。
        prev_key = prev_node.get_key()
        next_key = next_node.get_key()
        u = user_language_model.get_bigram_cost(prev_key, next_key)
        if u:
            return u

        id1 = prev_node.id
        id2 = next_node.id
        if id1 is None or id2 is None or id1 < 0 or id2 < 0:
            # print(f"BI MISS(NO KEY): {key1} {key2}")
            return BIGRAM_DEFAULT_COST
        score = system_language_model.find_bigram(id1, id2)

        # print(f"bigram: id1={id1}, id2={id2} score={score}")
        if score != 0.0:
            # print(f"BI HIT: {key1} {key2} -> {score}")
            return score
        else:
            # print(f"BI MISS: {key1} {key2}")
            return BIGRAM_DEFAULT_COST

    def get_bigram_cost(self, next_node, user_language_model, system_language_model):
        next_node_key = next_node.get_key()
        if next_node_key in self._bigram_cache:
            return self._bigram_cache[next_node_key]
        else:
            cost = self._calc_bigram_cost(self, next_node, user_language_model, system_language_model)
            self._bigram_cache[next_node_key] = cost
            return cost

 */
static float calc_bigram_cost(const akaza::Node &prev_node,
                              const akaza::Node &next_node,
                              const akaza::UserLanguageModel &ulm,
                              const akaza::SystemBigramLM &system_bigram_lm) {
    // self → node で処理する。
    auto prev_key = prev_node.get_key();
    auto next_key = next_node.get_key();
    auto u = ulm.get_bigram_cost(prev_key, next_key);
    if (u.has_value()) {
        return *u;
    }

    auto id1 = prev_node.get_word_id();
    auto id2 = next_node.get_word_id();
    if (id1 == akaza::UNKNOWN_WORD_ID || id2 == akaza::UNKNOWN_WORD_ID) {
        return system_bigram_lm.get_default_score();
    }

    auto score = system_bigram_lm.find_bigram(id1, id2);
    if (score != 0.0) {
        return score;
    } else {
        return system_bigram_lm.get_default_score();
    }
}

float akaza::Node::get_bigram_cost(const akaza::Node &next_node, const akaza::UserLanguageModel &ulm,
                                   const akaza::SystemBigramLM &system_bigram_lm) {
    auto next_node_key = next_node.get_key();
    if (_bigram_cache.count(next_node_key) > 0) {
        return _bigram_cache.at(next_node_key);
    } else {
        float cost = calc_bigram_cost(*this, next_node, ulm, system_bigram_lm);
        _bigram_cache[next_node_key] = cost;
        return cost;
    }
}
