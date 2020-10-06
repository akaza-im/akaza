#pragma once

#include <cstring>
#include <iostream>
#include <string>
#include <tuple>

#include "debug.h"
#include <marisa.h>

namespace akaza {

    const int UNKNOWN_WORD_ID = -1;

    class SystemUnigramLMBuilder {
    private:
        marisa::Keyset keyset;
        marisa::Trie trie;
    public:
        void add(const std::string &word, float score) {
            char buf[sizeof(float)];
            memcpy(buf, &score, sizeof(float));
            std::string key(word + "\xff" + std::string(buf, sizeof(float)));
            keyset.push_back(key.c_str(), key.size());
        }

        void save(const std::string &path) {
            trie.build(keyset);
            trie.save(path.c_str());
        }
    };

    class SystemUnigramLM {
    private:
        marisa::Trie trie;
    public:
        SystemUnigramLM() {}

        void load(const char *path) {
            trie.load(path);
            std::cout << path << " size: " << trie.size() << std::endl;
        }

        /**
         * @return {word_id}, {score}
         */
        std::tuple<int32_t, float> find_unigram(const std::string &word) {
            std::string query(word);
            query += "\xff"; // add marker

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (trie.predictive_search(agent)) {
                // dump_string(std::string(agent.key().ptr(), agent.key().length()));
                // std::cout << "HIT! " << std::endl;

                const char *p = agent.key().ptr() + query.size();
                float score = 0;
                std::memcpy(&score, p, sizeof(float));
                return std::tuple<int32_t, float>(int32_t(agent.key().id()), score);
            }
            return std::tuple<int32_t, float>(UNKNOWN_WORD_ID, 0);
        }
    };

    class SystemBigramLMBuilder {
    private:
        marisa::Keyset keyset;
        marisa::Trie trie;
    public:
        void add(int32_t word_id1, int32_t word_id2, float score) {
            // ここで処理する。
            std::string keybuf;
            uint8_t idbuf[sizeof(int32_t)];
            char scorebuf[sizeof(float)];

            // packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608
            std::memcpy(idbuf, &word_id1, sizeof(word_id1));
            keybuf += std::string(idbuf, idbuf + 3);
            std::memcpy(idbuf, &word_id2, sizeof(word_id2));
            keybuf += std::string(idbuf, idbuf + 3);

            // packed float  # score: 4 bytes
            std::memcpy(scorebuf, &score, sizeof(score));
            keybuf += std::string(scorebuf, scorebuf + 4);

            keyset.push_back(keybuf.c_str(), keybuf.size());
        }

        void save(const std::string &path) {
            trie.build(keyset);
            trie.save(path.c_str());
        }
    };

    class SystemBigramLM {
    private:
        marisa::Trie trie;
    public:
        SystemBigramLM() {
        }

        void load(const char *path) {
            trie.load(path);
        }

        float find_bigram(int32_t word_id1, int32_t word_id2) {
            uint32_t uword_id1 = word_id1;
            uint32_t uword_id2 = word_id2;
            uint8_t idbuf[4];
            std::string query;
            std::memcpy(idbuf, &uword_id1, sizeof(word_id1));
            query += std::string(idbuf, idbuf + 3);
            std::memcpy(idbuf, &uword_id2, sizeof(word_id2));
            query += std::string(idbuf, idbuf + 3);

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (trie.predictive_search(agent)) {
                const char *p = agent.key().ptr() + query.size();
                float score;
                std::memcpy(&score, p, sizeof(float));
                return score;
            }
            return 0;
        }
    };
} // namespace akaza
