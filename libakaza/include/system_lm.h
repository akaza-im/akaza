#ifndef LIBAKAZA_SYSTEM_LM_H_
#define LIBAKAZA_SYSTEM_LM_H_


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
        void add(const std::string &word, float score);

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

        void dump();

        void load(const char *path);

        /**
         * @return {word_id}, {score}
         */
        std::tuple<int32_t, float> find_unigram(const std::wstring &word) const;

        inline float get_default_cost() const {
            return -20.0; // log10(1e-20)
        }

        inline float get_default_cost_for_short() const {
            return -19.0; // log10(1e-19)
        }
    };


    class SystemBigramLMBuilder {
    private:
        marisa::Keyset keyset_;
        marisa::Trie trie_;
    public:
        void add(int32_t word_id1, int32_t word_id2, float score);

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

        float find_bigram(int32_t word_id1, int32_t word_id2) const;

        float get_default_score() const {
            return -20.0; // log10(1e-20)
        }
    };
} // namespace akaza

#endif // LIBAKAZA_SYSTEM_LM_H_
