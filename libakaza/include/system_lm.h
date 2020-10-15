#ifndef LIBAKAZA_SYSTEM_LM_H_
#define LIBAKAZA_SYSTEM_LM_H_


#include <cstring>
#include <iostream>
#include <string>
#include <tuple>

#include <marisa.h>

namespace akaza {

    const int UNKNOWN_WORD_ID = -1;

    class SystemUnigramLMBuilder {
    private:
        marisa::Keyset keyset_;
        marisa::Trie trie_;
    public:
        void add(const std::string &word, float score) {
            char buf[sizeof(float)];
            memcpy(buf, &score, sizeof(float));
            std::string key(word + "\xff" + std::string(buf, sizeof(float)));
            keyset_.push_back(key.c_str(), key.size());
        }

        void save(const std::string &path) {
            trie_.build(keyset_);
            trie_.save(path.c_str());
        }
    };

    class SystemUnigramLM {
    private:
        marisa::Trie trie_;
    public:
        SystemUnigramLM() {}

        ~SystemUnigramLM() {}

        std::size_t size() {
            return trie_.size();
        }

        void load(const char *path);

        /**
         * @return {word_id}, {score}
         */
        std::tuple<int32_t, float> find_unigram(const std::wstring &word) const;

        inline float get_default_cost() const {
            return -20.0; // log10(1e-20)
        }
    };


    class SystemBigramLMBuilder {
    private:
        marisa::Keyset keyset_;
        marisa::Trie trie_;
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

            keyset_.push_back(keybuf.c_str(), keybuf.size());
        }

        void save(const std::string &path) {
            trie_.build(keyset_);
            trie_.save(path.c_str());
        }
    };

    class SystemBigramLM {
    private:
        marisa::Trie trie_;
    public:
        SystemBigramLM() {
        }

        std::size_t size() {
            return trie_.size();
        }

        void load(const char *path);

        float find_bigram(int32_t word_id1, int32_t word_id2) const {
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

            while (trie_.predictive_search(agent)) {
                const char *p = agent.key().ptr() + query.size();
                float score;
                std::memcpy(&score, p, sizeof(float));
                return score;
            }
            return 0;
        }

        float get_default_score() const {
            return -20.0; // log10(1e-20)
        }
    };
} // namespace akaza

#endif // LIBAKAZA_SYSTEM_LM_H_
