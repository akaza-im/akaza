#include "../include/akaza.h"
#include <codecvt>
#include "split.h"

inline int my_atoi(const std::wstring &s) {
    std::wstringstream ss(s);
    int n;
    ss >> n;
    return n;
}

void akaza::UserLanguageModel::read(const std::string &path, bool is_unigram, int &c, int &v,
                                    std::map<std::wstring, int> &map) {
/*
        word_data = {}
        with open(path) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    key, count = m
                    count = int(count)
                    word_data[key] = count
                    if is_unigram:
                        kanji, kana = key.split('/')
                        self.unigram_kanas.add(kana)
                    V += 1
                    C += count
        return V, C, word_data
 */
    c = 0;
    v = 0;

    std::wifstream ifs(path, std::ifstream::in);
    ifs.imbue(std::locale(std::locale(), new std::codecvt_utf8<wchar_t>));
    std::wstring line;
    while (std::getline(ifs, line)) {
        bool splitted;
        wchar_t sp = L' ';
        std::tuple<std::wstring, std::wstring> m = split2(line, sp, splitted);
        if (!splitted) {
            continue;
        }
        std::wstring key = std::get<0>(m);
        int count = my_atoi(std::get<1>(m));
        map[key] = count;
        if (is_unigram) {
            std::wstring kana = std::get<1>(split2(line, L'/', splitted));
            if (splitted) {
                unigram_kanas_.insert(kana);
            }
        }
        v += 1;
        c += count;
    }
}

/*
def add_entry(self, nodes: List[Node]):
    # unigram
    for node in nodes:
        key = node.get_key()

        self.logger.info(f"add user_language_model entry: key={key}")

        if key not in self.unigram:
            self.unigram_C += 1
        self.unigram_V += 1
        kanji, kana = key.split('/')
        self.unigram_kanas.add(kana)
        self.unigram[key] = self.unigram.get(key, 0) + 1

    # bigram
    for i in range(1, len(nodes)):
        node1 = nodes[i - 1]
        node2 = nodes[i]
        key = node1.get_key() + "\t" + node2.get_key()
        if key not in self.bigram:
            self.bigram_C += 1
        self.bigram_V += 1
        self.bigram[key] = self.bigram.get(key, 0) + 1

    self.need_save = True
 */
void akaza::UserLanguageModel::add_entry(std::vector<Node> nodes) {
    // unigram
    for (const akaza::Node &node: nodes) {
        std::wstring key = node.get_key();
        if (unigram_.count(key) == 0) {
            unigram_C_ += 1;
        }
        unigram_V_ += 1;
        bool splitted;
        std::wstring kana = std::get<1>(split2(key, L'/', splitted));
        unigram_kanas_.insert(kana);
        unigram_[key] = unigram_.count(key) > 0 ? unigram_[key] + 1 : 1;
    }

    // bigram
    for (int i = 1; i < nodes.size(); i++) {
        const akaza::Node &node1 = nodes[i - 1];
        const akaza::Node &node2 = nodes[i];

        std::wstring key = node1.get_key() + L"\t" + node2.get_key();
        if (bigram_.count(key) == 0) {
            bigram_C_ += 1;
        }
        bigram_V_ += 1;
        bigram_[key] = unigram_.count(key) > 0 ? unigram_[key] + 1 : 1;
    }

    need_save_ = true;
}

std::optional<float> akaza::UserLanguageModel::get_unigram_cost(const std::wstring &key) const {
    if (unigram_.count(key) > 0) {
        int count = unigram_.at(key);
        return std::log10((float(count) + alpha_) / float(unigram_C_) + alpha_ * float(unigram_V_));
    }
    return {};
}

std::optional<float>
akaza::UserLanguageModel::get_bigram_cost(const std::wstring &key1, const std::wstring &key2) const {
    auto key = key1 + L"\t" + key2;
    if (bigram_.count(key) > 0) {
        int count = bigram_.at(key);
        return std::log10((float(count) + alpha_) / (float(bigram_C_) + alpha_ * float(bigram_V_)));
    } else {
        return {};
    }
}

void akaza::UserLanguageModel::save_file(const std::string &path, const std::map<std::wstring, int> &map) {
    std::wofstream ofs(path + ".tmp", std::ofstream::out);
    for (const auto&[words, count] : map) {
        ofs << words << " " << count << std::endl;
    }
    ofs.close();
    rename(path.c_str(), (path + ".tmp").c_str());
}
