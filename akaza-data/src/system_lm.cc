#include <string>
#include <iostream>
#include <cstring>

#include <marisa.h>

namespace akaza {
    struct UnigramEntry {
        int id;
        float score;
    };

    class SystemLM {
    private:
        marisa::Trie unigram_trie;
        marisa::Trie bigram_trie;
    public:
        SystemLM() {
        }

        void load(std::string unigram_path, std::string bigram_path) {
            unigram_trie.load(unigram_path.c_str());
            std::cout << unigram_path << ": " << unigram_trie.num_keys() << std::endl;
            bigram_trie.load(bigram_path.c_str());
            std::cout << bigram_path << ": " << bigram_trie.num_keys() << std::endl;
        }

        /**
         * @return hit or not
         */
        bool find_bigram(uint32_t word_id1, uint32_t word_id2, float &score) {
            uint8_t idbuf[4];
            std::string query;
            std::memcpy(idbuf, &word_id1, sizeof(word_id1));
            query += std::string(idbuf, idbuf+3);
            std::memcpy(idbuf, &word_id2, sizeof(word_id2));
            query += std::string(idbuf, idbuf+3);

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (bigram_trie.predictive_search(agent)) {
                const char * p = agent.key().ptr() + query.size();
                std::memcpy(&score, p, sizeof(float));
                return true;
            }
            return false;
        }

        void dump_bigram() {
            marisa::Agent agent;
            agent.set_query("");
            while (bigram_trie.predictive_search(agent)) {
                std::cout.write(agent.key().ptr(), agent.key().length());
                std::cout << ": " << agent.key().id() << std::endl;
            }
        }

        void dump_unigram() {
            marisa::Agent agent;
            agent.set_query("");
            while (unigram_trie.predictive_search(agent)) {
                std::cout.write(agent.key().ptr(), agent.key().length());
                std::cout << ": " << agent.key().id() << std::endl;
            }
        }

        /**
         */
        bool find_unigram(std::string word, uint32_t & id, float &score) {
            std::string query(word);
            query += "\xff"; // add marker

            marisa::Agent agent;
            agent.set_query(word.c_str(), word.size());

            while (unigram_trie.predictive_search(agent)) {
                const char * p = agent.key().ptr() + query.size();
                id = uint8_t(p[0]) + (uint8_t(p[1])<<8) + (uint8_t(p[2])<<16);
                std::memcpy(&score, p+3, sizeof(float));
                return true;
            }
            return false;
        }
    };
}

#ifdef AKAZA_TEST

static int get_id(akaza::SystemLM &lm, std::string word) {
    uint32_t id = 0;
    float score;
    bool hit = lm.find_unigram(word, id, score);
    std::cout << "hit=" << hit << " id=" << id << " score=" << score << " word=" << word << std::endl;
    return id;
}

int main() {
    akaza::SystemLM lm;
    lm.load("akaza_data/data/lm_v2_1gram.trie", "akaza_data/data/lm_v2_2gram.trie");

    // get_id(lm, "堂嶋/どうじま");

    int id_watasi = get_id(lm, "私/わたし");
    int id_ha = get_id(lm, "は/は");
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
        float score = 0;
        bool hit = lm.find_bigram(id_watasi, id_ja, score);
        std::cout << "hit=" << hit << " score=" << score << std::endl;
    }
}

#endif
