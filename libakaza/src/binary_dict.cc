#include "../include/binary_dict.h"
#include "debug_log.h"
#include <codecvt>
#include <locale>
#include <iostream>
#include <sstream>

static std::vector<std::wstring> split(const std::wstring &s) {
    std::vector<std::wstring> elems;
    std::wstringstream ss(s);
    std::wstring item;
    while (getline(ss, item, L'/')) {
        if (!item.empty()) {
            elems.push_back(item);
        }
    }
    return elems;
}


void akaza::BinaryDict::load(const std::string &dict_path) {
    dict_trie_.load(dict_path.c_str());
    D(std::cout << "Loading BinaryDict: " << dict_path << ": " << dict_trie_.num_keys()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}

std::vector<std::wstring> akaza::BinaryDict::find_kanjis(const std::wstring &word) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;

    std::string query(cnv.to_bytes(word));
    query += "\xff"; // add marker

    marisa::Agent agent;
    agent.set_query(query.c_str(), query.size());

    while (dict_trie_.predictive_search(agent)) {
        std::string kanjis = std::string(agent.key().ptr() + query.size(), agent.key().length() - query.size());
        return split(cnv.from_bytes(kanjis));
    }
    return std::vector<std::wstring>();
}

void akaza::BinaryDict::save(const std::string& dict_path) {
    dict_trie_.save(dict_path.c_str());
    std::cout << "[Save] " << dict_path << ": " << dict_trie_.num_keys() << std::endl;
}
