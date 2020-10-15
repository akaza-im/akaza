#pragma once

#include <cstring>
#include <iostream>
#include <string>
#include <vector>
#include <sstream>
#include <tuple>

#include <marisa.h>


namespace akaza {
    class BinaryDict {
    private:
        marisa::Trie dict_trie;

    public:
        BinaryDict() {}

        size_t size() {
            return dict_trie.size();
        }

        void load(const std::string &dict_path);

        void save(std::string dict_path) {
            dict_trie.save(dict_path.c_str());
            std::cout << "[Save] " << dict_path << ": " << dict_trie.num_keys() << std::endl;
        }

        void build_by_keyset(marisa::Keyset &keyset) {
            dict_trie.build(keyset);
        }

        // vector of "とくひろ" => "徳宏/徳大/徳寛/督弘"
        void build(std::vector<std::tuple<std::string, std::string>> data) {
            marisa::Keyset keyset;
            for (auto &d: data) {
                std::string yomi = std::get<0>(d);
                std::string kanjis = std::get<1>(d);
                keyset.push_back((yomi + "\xff" + kanjis).c_str());
            }
            this->build_by_keyset(keyset);
        }

        std::vector<std::wstring> find_kanjis(const std::wstring &word);

    };
} // namespace akaza
