#ifndef LIBAKAZA_USER_LANGUAGE_MODEL_H
#define LIBAKAZA_USER_LANGUAGE_MODEL_H

#include <string>
#include <set>
#include <map>
#include <fstream>
#include <cmath>
#include <vector>
#include <unordered_map>
#include <unordered_set>

namespace akaza {
    class Node;

    class UserLanguageModel {

    private:
        std::string unigram_path_;
        std::string bigram_path_;

        bool need_save_ = false;

        std::unordered_set<std::wstring> unigram_kanas_;

        // 単語数
        int unigram_C_ = 0;
        // 総単語出現数
        int unigram_V_ = 0;
        std::unordered_map<std::wstring, int> unigram_;
        int bigram_C_ = 0;
        int bigram_V_ = 0;
        std::unordered_map<std::wstring, int> bigram_;

        float alpha_ = 0.00001;

        void read(const std::string &path, bool is_unigram, int &c, int &v, std::unordered_map<std::wstring, int> &map);

        static void save_file(const std::string &path, const std::unordered_map<std::wstring, int> &map);

    public:
        UserLanguageModel(const std::string &unigram_path, const std::string &bigram_path) {
            this->unigram_path_ = unigram_path;
            this->bigram_path_ = bigram_path;
        }

        size_t size_unigram() {
            return unigram_.size();
        }

        size_t size_bigram() {
            return bigram_.size();
        }

        void load_unigram() {
            read(unigram_path_, true, unigram_C_, unigram_V_, unigram_);
        }

        void load_bigram() {
            read(bigram_path_, false, bigram_C_, bigram_V_, bigram_);
        }

        void add_entry(std::vector<Node> nodes);

        std::optional<float> get_unigram_cost(const std::wstring &key) const;

        bool has_unigram_cost_by_yomi(const std::wstring &yomi) {
            return unigram_kanas_.count(yomi) > 0;
        }

        std::optional<float> get_bigram_cost(const std::wstring &key1, const std::wstring &key2) const;

        void save() {
            need_save_ = false;

            save_file(unigram_path_, unigram_);
            save_file(bigram_path_, bigram_);
        }

        bool should_save() {
            return need_save_;
        }

    };
}

#endif //LIBAKAZA_USER_LANGUAGE_MODEL_H
