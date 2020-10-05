#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../src/system_lm.h"

static int get_id(akaza::SystemLM &lm, std::string word) {
    auto tuple = lm.find_unigram(word);
    uint32_t id = std::get<0>(tuple);
    float score = std::get<1>(tuple);
    std::cout << " id=" << id << " score=" << score << " word=" << word << std::endl;
    return id;
}

int main() {
    akaza::SystemLM lm;
    lm.load("akaza_data/data/lm_v2_1gram.trie", "akaza_data/data/lm_v2_2gram.trie");
    get_id(lm, "三/み");

    // get_id(lm, "堂嶋/どうじま");

    int id_watasi = get_id(lm, "私/わたし");
    int id_ha = get_id(lm, "は/は");
    ok(id_ha > 0);
    int id_ja = get_id(lm, "じゃ/じゃ");
    // lm.dump_unigram();
    // lm.dump_bigram();

/*
    {
        float score = 0;
        bool hit = lm.find_bigram(id_watasi, id_ha, score);
        std::cout << "hit=" << hit << " score=" << score << std::endl;
    } */
    {
        float score = lm.find_bigram(id_watasi, id_ja);
        std::cout << " score=" << score << std::endl;
        ok(score < 0);
    }
    done_testing();
}
