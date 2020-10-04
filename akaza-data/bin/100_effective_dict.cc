#include <iostream>
#include <fstream>
#include <sstream>
#include <map>
#include <cstring>
#include <vector>

#include <marisa.h>

static std::vector<std::string> split(const std::string &s) {
    std::vector<std::string> elems;
    std::stringstream ss(s);
    size_t n = s.find_first_of(" ");
    if (n == std::string::npos) {
        return elems;
    }
    elems.push_back(s.substr(0, n));
    elems.push_back(s.substr(n+1));
    return elems;
}

void make_system_dict(std::string ifname, std::string ofname) {
    std::cout << "[100_effective_dict.cc] " << ifname << std::endl;

    std::ifstream ifs(ifname, std::ifstream::in);
    std::string word;
    std::string kanjis;
    marisa::Keyset keyset;

    std::string buffer;

    while (std::getline(ifs, buffer)) {
        auto data = split(buffer);
        if (data.size() != 2) {
            std::cout << buffer << std::endl;
            break;
        }
        word = data[0];
        kanjis = data[1];

        std::string keybuf(word);

        // std::cout << word << "\t--- " << kanjis << std::endl;

        // marker
        keybuf += "\xff";
        keybuf += kanjis;

        keyset.push_back(keybuf.c_str(), keybuf.size());
    }

    marisa::Trie trie;
    trie.build(keyset);
    trie.save(ofname.c_str());
}

int main() {
    make_system_dict("work/jawiki.system_dict.txt", "akaza_data/data/system_dict.trie");
    make_system_dict("work/jawiki.single_term.txt", "akaza_data/data/single_term.trie");
    return 0;
}
