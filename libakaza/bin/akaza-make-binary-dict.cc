#include <iostream>
#include <fstream>
#include <sstream>
#include <map>
#include <cstring>
#include <vector>

#include <marisa.h>
#include "../include/akaza.h"

static std::vector<std::string> split(const std::string &s) {
    std::vector<std::string> elems;
    std::stringstream ss(s);
    size_t n = s.find_first_of(' ');
    if (n == std::string::npos) {
        return elems;
    }
    elems.push_back(s.substr(0, n));
    elems.push_back(s.substr(n + 1));
    return elems;
}

static void make_binary_dict(const std::string &ifname, const std::string &ofname) {
    std::cout << ifname << " " << __FILE__ << ":" << __LINE__ << std::endl;

    std::ifstream ifs(ifname, std::ifstream::in);

    std::string buffer;
    std::vector<std::tuple<std::string, std::string>> set;
    akaza::BinaryDict builder;

    while (std::getline(ifs, buffer)) {
        auto data = split(buffer);
        if (data.size() != 2) {
            std::cout << buffer << std::endl;
            break;
        }
        auto word = data[0];
        auto kanjis = data[1];

        set.emplace_back(word, kanjis);
    }

    builder.build(set);
    builder.save(ofname);
}

int main(int argc, char **argv) {
    const char *txtfile = argv[1];
    const char *triefile = argv[2];
    make_binary_dict(txtfile, triefile);
    return 0;
}
