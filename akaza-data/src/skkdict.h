#include <string>
#include <iostream>
#include <sstream>
#include <map>
#include <fstream>
#include <vector>

#pragma once

/*
def parse_skkdict(path: str, encoding: str = 'euc-jp'):
    nasi: Dict[str, List[str]] = {}
    ari: Dict[str, List[str]] = {}
    target = ari

    with open(path, 'r', encoding=encoding) as fp:
        for line in fp:
            if line == ";; okuri-ari entries.\n":
                target = ari
            if line == ";; okuri-nasi entries.\n":
                target = nasi
            if line.startswith(';;'):
                continue

            m = line.strip().split(' ', 1)
            if len(m) == 2:
                yomi: str = m[0]
                kanjis: List[str] = m[1].lstrip('/').rstrip('/').split('/')
                kanjis = [re.sub(';.*', '', k) for k in kanjis]

                target[yomi] = kanjis

    return ari, nasi
*/
namespace akaza {
    /**
     * Parse SKK dictionary.
     * This only supports UTF-8 dictionary.
     */
    std::tuple<
        std::map<std::string, std::vector<std::string>>,
        std::map<std::string, std::vector<std::string>>
     > parse_skkdict(std::string path) {
        std::ifstream ifs(path, std::ifstream::in);
        std::string line;
        std::map<std::string, std::vector<std::string>> ari;
        std::map<std::string, std::vector<std::string>> nasi;
        std::map<std::string, std::vector<std::string>> * target = &ari;

        while (std::getline(ifs, line)) {
            if (line.rfind(";; okuri-ari entries.") == 0) { // okuri-ari mode
                target = &ari;
                continue;
            }
            if (line.rfind(";; okuri-nasi entries.") == 0) { // okuri-nasi mode
                target = &nasi;
                continue;
            }
            if (line.rfind(";;") == 0) { // skip comment
                continue;
            }
            std::size_t pos = line.find(' ');
            std::string yomi = line.substr(0, pos);
            std::cout << yomi << std::endl;

            std::stringstream ss(line.substr(pos+1));
            std::vector<std::string> kanjis;
            std::string kanji;
            while (getline(ss, kanji, '/')) {
                if (!kanji.empty()) {
                    kanjis.push_back(kanji);
                }
            }
            (*target)[yomi] = kanjis;
        }
        return std::make_tuple(ari, nasi);
    }
}
