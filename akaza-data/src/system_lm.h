#include <string>
#include <iostream>
#include <cstring>
#include <tuple>

#include "debug.h"
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
            if (unigram_path == bigram_path) {
                throw "Path conflict";
            }

            unigram_trie.load(unigram_path.c_str());
            std::cout << unigram_path << ": " << unigram_trie.num_keys() << std::endl;
            bigram_trie.load(bigram_path.c_str());
            std::cout << bigram_path << ": " << bigram_trie.num_keys() << std::endl;
        }

        /**
         * @return hit or not
         */
        float find_bigram(int32_t word_id1, int32_t word_id2) {
            uint32_t uword_id1 = word_id1;
            uint32_t uword_id2 = word_id2;
            uint8_t idbuf[4];
            std::string query;
            std::memcpy(idbuf, &uword_id1, sizeof(word_id1));
            query += std::string(idbuf, idbuf+3);
            std::memcpy(idbuf, &uword_id2, sizeof(word_id2));
            query += std::string(idbuf, idbuf+3);

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (bigram_trie.predictive_search(agent)) {
                const char * p = agent.key().ptr() + query.size();
                float score;
                std::memcpy(&score, p, sizeof(float));
                return score;
            }
            return 0;
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
         * @return {word_id}, {score}
         */
        std::tuple<int32_t, float> find_unigram(std::string word) {
            std::string query(word);
            query += "\xff"; // add marker

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (unigram_trie.predictive_search(agent)) {
                // dump_string(std::string(agent.key().ptr(), agent.key().length()));

                const char * p = agent.key().ptr() + query.size();
                int32_t id = uint8_t(p[0]) + (uint8_t(p[1])<<8) + (uint8_t(p[2])<<16);
                float score=0;
                std::memcpy(&score, p+3, sizeof(float));
                return std::tuple<int32_t, float>(id, score);
            }
            return std::tuple<int32_t, float>(-1, 0);
        }
    };
}

