#ifndef LIBAKAZA_NODE_H
#define LIBAKAZA_NODE_H

#include <map>
#include <string>
#include <memory>

namespace akaza {

    class UserLanguageModel;
    class SystemUnigramLM;
    class SystemBigramLM;

    namespace tinylisp {
        class TinyLisp;
    }

    class Node {
    private:
        int start_pos_;
        std::wstring yomi_;
        std::wstring word_;
        std::wstring key_;
        std::shared_ptr<Node> prev_;
        float cost_;
        int32_t word_id_;
        std::map<std::wstring, float> bigram_cache_;
    public:
        Node(int start_pos, const std::wstring &yomi, const std::wstring &word);

        std::wstring get_key() const {
            return this->key_;
        }

        bool is_bos() const {
            return word_ == L"__BOS__";
        }

        bool is_eos() const {
            return word_ == L"__EOS__";
        }

        std::wstring surface(const akaza::tinylisp::TinyLisp &tinyLisp) const;


        float calc_node_cost(const akaza::UserLanguageModel &user_language_model, const akaza::SystemUnigramLM &ulm);

        float get_bigram_cost(const akaza::Node &next_node, const UserLanguageModel &ulm,
                              const SystemBigramLM &system_bigram_lm);

        int32_t get_word_id() const {
            return word_id_;
        }

        std::wstring get_yomi() const {
            return yomi_;
        }

        std::wstring get_word() const {
            return word_;
        }

        float get_cost() const {
            return cost_;
        }

        void set_cost(float cost) {
            cost_ = cost;
        }

        int get_start_pos() const {
            return start_pos_;
        }

        std::shared_ptr<Node> get_prev() const {
            return prev_;
        }

        void set_prev(std::shared_ptr<Node> &prev);

        bool operator==(const Node &node);
        bool operator!=(const Node &node);
    };

    std::shared_ptr<Node> create_bos_node();

    std::shared_ptr<Node> create_eos_node(int start_pos);

}

#endif //LIBAKAZA_NODE_H
