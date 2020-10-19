#ifndef LIBAKAZA_NODE_H
#define LIBAKAZA_NODE_H

#include <map>
#include <string>
#include <memory>
#include <unordered_map>

namespace akaza {

    class UserLanguageModel;

    class SystemUnigramLM;

    class SystemBigramLM;

    namespace tinylisp {
        class TinyLisp;
    }

    class Node {
    private:
        const int start_pos_;
        const std::wstring yomi_;
        const std::wstring word_;
        const std::wstring key_;
        const bool is_bos_;
        const bool is_eos_;
        const int32_t system_word_id_;
        const float system_unigram_cost_;

        float total_cost_; // unigram cost + bigram cost + previous cost

        std::shared_ptr<Node> prev_;
        std::unordered_map<std::wstring, float> bigram_cache_;
    public:
        Node(int start_pos, const std::wstring &yomi, const std::wstring &word, const std::wstring &key,
             bool is_bos, bool is_eos,
             int32_t system_word_id, float system_unigram_cost) :
                start_pos_(start_pos),
                yomi_(yomi),
                word_(word),
                key_(key),
                is_bos_(is_bos), is_eos_(is_eos),
                system_word_id_(system_word_id),
                system_unigram_cost_(system_unigram_cost) {
        }

        inline std::wstring get_key() const {
            return this->key_;
        }

        bool is_bos() const {
            return is_bos_;
        }

        bool is_eos() const {
            return is_eos_;
        }

        std::wstring surface(const akaza::tinylisp::TinyLisp &tinyLisp) const;


        float calc_node_cost(const akaza::UserLanguageModel &user_language_model, const akaza::SystemUnigramLM &ulm);

        float get_bigram_cost(const akaza::Node &next_node, const UserLanguageModel &ulm,
                              const SystemBigramLM &system_bigram_lm);

        float get_bigram_cost_from_cache(const akaza::Node &next_node,
                                         const akaza::SystemBigramLM &system_bigram_lm) const;

        inline int32_t get_word_id() const {
            return system_word_id_;
        }

        inline std::wstring get_yomi() const {
            return yomi_;
        }

        inline std::wstring get_word() const {
            return word_;
        }

        inline float get_total_cost() const {
            return total_cost_;
        }

        inline void set_total_cost(float cost) {
            total_cost_ = cost;
        }

        inline int get_start_pos() const {
            return start_pos_;
        }

        inline std::shared_ptr<Node> get_prev() const {
            return prev_;
        }

        void set_prev(std::shared_ptr<Node> &prev);

        bool operator==(const Node &node);

        bool operator!=(const Node &node);

        friend class Graph;
    };

    std::shared_ptr<Node> create_bos_node();

    std::shared_ptr<Node> create_eos_node(int start_pos);

    std::shared_ptr<Node>
    create_node(std::shared_ptr<akaza::SystemUnigramLM> &system_unigram_lm, int start_pos, const std::wstring &yomi,
                const std::wstring &kanji);

}

#endif //LIBAKAZA_NODE_H
