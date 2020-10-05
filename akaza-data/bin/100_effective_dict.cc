#include <iostream>
#include <fstream>
#include <sstream>
#include <map>
#include <cstring>
#include <vector>

#include <marisa.h>
#include "../src/binary_dict.h"

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

    std::string buffer;
    std::vector<std::tuple<std::string, std::string>> set;

    while (std::getline(ifs, buffer)) {
        auto data = split(buffer);
        if (data.size() != 2) {
            std::cout << buffer << std::endl;
            break;
        }
        auto word = data[0];
        auto kanjis = data[1];

        set.push_back(std::make_tuple(word, kanjis));
    }

    akaza::BinaryDict dict;
    dict.build(set);
    dict.save(ofname);
}

int main() {
    make_system_dict("work/jawiki.system_dict.txt", "akaza_data/data/system_dict.trie");
    make_system_dict("work/jawiki.single_term.txt", "akaza_data/data/single_term.trie");
    return 0;
}
