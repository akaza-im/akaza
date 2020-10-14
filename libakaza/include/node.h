#ifndef LIBAKAZA_NODE_H
#define LIBAKAZA_NODE_H

#include <map>
#include <string>
#include <assert.h>

namespace akaza {

    class UserLanguageModel;

    class Node {
    private:
        int start_pos;
        std::string yomi;
        std::string word;
        std::string _key;
        std::shared_ptr<Node> _prev;
        float _cost;
        int32_t word_id;
        std::map<std::string, float> _bigram_cache;
    public:
        Node(int start_pos, const std::string &yomi, const std::string &word) {
            this->start_pos = start_pos;
            this->yomi = yomi;
            this->word = word;
            if (word == "__EOS__") {
                // return '__EOS__'  // わざと使わない。__EOS__ 考慮すると変換精度が落ちるので。。今は使わない。
                // うまく使えることが確認できれば、__EOS__/__EOS__ にする。
                this->_key = "__EOS__";
            } else {
                this->_key = word + "/" + yomi;
            }
            this->_cost = 0;
            this->word_id = -1;
        }

        std::string get_key() const {
            return this->_key;
        }

        bool is_bos() const {
            return word == "__BOS__";
        }

        bool is_eos() const {
            return word == "__EOS__";
        }

        std::string surface(const akaza::tinylisp::TinyLisp &tinyLisp) const {
            if (word.size() > 0 && word[0] == '(') {
                return tinyLisp.run(word);
            } else {
                return word;
            }
        }


        float calc_node_cost(const akaza::UserLanguageModel &user_language_model, const akaza::SystemUnigramLM &ulm);

        float get_bigram_cost(const akaza::Node &next_node, const UserLanguageModel &ulm,
                              const SystemBigramLM &system_bigram_lm);

        int32_t get_word_id() const {
            return word_id;
        }

        std::string get_yomi() const {
            return yomi;
        }

        std::string get_word() const {
            return word;
        }

        float get_cost() const {
            return _cost;
        }

        void set_cost(float cost) {
            _cost = cost;
        }

        int get_start_pos() const {
            return start_pos;
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
