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
    auto search = unigram_.find(key);
    if (search != unigram_.cend()) {
        int count = search->second;
        return std::log10((float(count) + alpha_) / float(unigram_C_) + alpha_ * float(unigram_V_));
    }
    return {};
}

std::optional<float>
akaza::UserLanguageModel::get_bigram_cost(const std::wstring &key1, const std::wstring &key2) const {
    auto key = key1 + L"\t" + key2;
    auto search = bigram_.find(key);
    if (search != bigram_.cend()) {
        int count = search->second;
        return std::log10((float(count) + alpha_) / (float(bigram_C_) + alpha_ * float(bigram_V_)));
    } else {
        return {};
    }
}

void akaza::UserLanguageModel::save_file(const std::string &path, const std::map<std::wstring, int> &map) {
    std::string tmppath(path + ".tmp");
    std::wofstream ofs(tmppath, std::ofstream::out);
    ofs.imbue(std::locale(std::locale(), new std::codecvt_utf8<wchar_t>));

    for (const auto&[words, count] : map) {
        ofs << words << " " << count << std::endl;
    }
    ofs.close();

    int status = std::rename(tmppath.c_str(), path.c_str());
    if (status != 0) {
        std::string err = strerror(errno);
        throw std::runtime_error(err + " : " + path + " (Cannot write user language model)");
    }
}
