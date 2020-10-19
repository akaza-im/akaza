#ifndef LIBAKAZA_USER_LANGUAGE_MODEL_H
#define LIBAKAZA_USER_LANGUAGE_MODEL_H

#include <string>
#include <set>
#include <map>
#include <fstream>
#include <cmath>
#include <vector>

namespace akaza {
    class Node;

    class UserLanguageModel {

    private:
        std::string unigram_path_;
        std::string bigram_path_;

        bool need_save_ = false;

        std::set<std::wstring> unigram_kanas_;

        // 単語数
        int unigram_C_ = 0;
        // 総単語出現数
        int unigram_V_ = 0;
        std::map<std::wstring, int> unigram_;
        int bigram_C_ = 0;
        int bigram_V_ = 0;
        std::map<std::wstring, int> bigram_;

        float alpha_ = 0.00001;

        void read(const std::string &path, bool is_unigram, int &c, int &v, std::map<std::wstring, int> &map);

        static void save_file(const std::string &path, const std::map<std::wstring, int> &map);

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

/*
    def get_unigram_cost(self, key: str) -> Optional[float]:
        if key in self.unigram:
            count = self.unigram[key]
            return math.log10((count + ALPHA) / (self.unigram_C + ALPHA * self.unigram_V))
        return None

    def has_unigram_cost_by_yomi(self, yomi: str) -> bool:
        return yomi in self.unigram_kanas

    def get_bigram_cost(self, key1: str, key2: str) -> Optional[float]:
        key = f"{key1}\t{key2}"
        if key in self.bigram:
            count = self.bigram[key]
            return math.log10((count + ALPHA) / (self.bigram_C + ALPHA * self.bigram_V))
        return None
 */

/*
    def save(self):
        if not self.need_save:
            self.logger.debug("Skip saving user_language_mdel.")
            return

        self.need_save = False
        self.logger.info("Writing user_language_model")
        with atomic_write(self.unigram_path(), overwrite=True) as f:
            for words in sorted(self.unigram.keys()):
                count = self.unigram[words]
                f.write(f"{words} {count}\n")

        with atomic_write(self.bigram_path(), overwrite=True) as f:
            for words in sorted(self.bigram.keys()):
                count = self.bigram[words]
                f.write(f"{words} {count}\n")

        self.logger.info(f"SAVED {self.path}")
 */
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
