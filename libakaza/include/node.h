#ifndef LIBAKAZA_NODE_H
#define LIBAKAZA_NODE_H

#include <map>
#include <string>
#include <assert.h>
#include <codecvt>
#include <locale>

namespace akaza {

    class UserLanguageModel;

    class Node {
    private:
        int start_pos_;
        std::wstring yomi_;
        std::string word_;
        std::string key_;
        std::shared_ptr<Node> _prev;
        float cost_;
        int32_t word_id_;
        std::map<std::wstring, float> _bigram_cache;
    public:
        Node(int start_pos, const std::string &yomi, const std::string &word);

        std::string get_key() const {
            return this->key_;
        }

        bool is_bos() const {
            return word_ == "__BOS__";
        }

        bool is_eos() const {
            return word_ == "__EOS__";
        }

        std::string surface(const akaza::tinylisp::TinyLisp &tinyLisp) const {
            std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
            if (word_.size() > 0 && word_[0] == '(') {
                return cnv.to_bytes(tinyLisp.run(cnv.from_bytes(word_)));
            } else {
                return word_;
            }
        }


        float calc_node_cost(const akaza::UserLanguageModel &user_language_model, const akaza::SystemUnigramLM &ulm);

        float get_bigram_cost(const akaza::Node &next_node, const UserLanguageModel &ulm,
                              const SystemBigramLM &system_bigram_lm);

        int32_t get_word_id() const {
            return word_id_;
        }

        std::wstring get_yomi() const {
            return yomi_;
        }

        std::string get_word() const {
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
            return _prev;
        }

        void set_prev(std::shared_ptr<Node> &prev);

        bool operator==(const Node &node);
        bool operator!=(const Node &node);
    };

    std::shared_ptr<Node> create_bos_node();

    std::shared_ptr<Node> create_eos_node(int start_pos);

}

#endif //LIBAKAZA_NODE_H
