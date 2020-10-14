#ifndef LIBAKAZA_USER_LANGUAGE_MODEL_H
#define LIBAKAZA_USER_LANGUAGE_MODEL_H

#include <string>
#include <set>
#include <map>
#include <fstream>
#include <cmath>

#include "node.h"

namespace akaza {
    class UserLanguageModel {

    private:
        std::string unigram_path;
        std::string bigram_path;

        bool need_save = false;

        std::set<std::string> unigram_kanas;

        // 単語数
        int unigram_C = 0;
        // 総単語出現数
        int unigram_V = 0;
        std::map<std::string, int> unigram;
        int bigram_C = 0;
        int bigram_V = 0;
        std::map<std::string, int> bigram;

        float alpha = 0.00001;

        void read(const std::string &path, bool is_unigram, int &c, int &v, std::map<std::string, int> &map);

        void save_file(const std::string &path, const std::map<std::string, int> &map) {
            std::ofstream ofs(path + ".tmp", std::ofstream::out);
            for (const auto&[words, count] : map) {
                ofs << words << " " << count << std::endl;
            }
            ofs.close();
            rename(path.c_str(), (path + ".tmp").c_str());
        }

    public:
        UserLanguageModel(const std::string &unigram_path, const std::string &bigram_path) {
            this->unigram_path = unigram_path;
            this->bigram_path = bigram_path;
        }

        size_t size_unigram() {
            return unigram.size();
        }

        size_t size_bigram() {
            return bigram.size();
        }

        void load_unigram() {
            read(unigram_path, true, unigram_C, unigram_V, unigram);
        }

        void load_bigram() {
            read(bigram_path, false, bigram_C, bigram_V, bigram);
        }

        void add_entry(std::vector<Node> nodes);

        std::optional<float> get_unigram_cost(const std::string &key) const {
            if (unigram.count(key) > 0) {
                auto count = unigram.at(key);
                return std::log10((count + alpha) / unigram_C + alpha * unigram_V);
            }
            return {};
        }

        bool has_unigram_cost_by_yomi(const std::string &yomi) {
            return unigram_kanas.count(yomi) > 0;
        }

        std::optional<float> get_bigram_cost(const std::string &key1, const std::string &key2) const {
            auto key = key1 + "\t" + key2;
            if (bigram.count(key) > 0) {
                auto count = bigram.at(key);
                return std::log10((count + alpha) / (bigram_C + alpha * bigram_V));
            } else {
                return {};
            }
        }

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
            need_save = false;

            save_file(unigram_path, unigram);
            save_file(bigram_path, bigram);
        }

        bool should_save() {
            return need_save;
        }

    };
}

#endif //LIBAKAZA_USER_LANGUAGE_MODEL_H
