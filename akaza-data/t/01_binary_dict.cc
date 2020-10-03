#include "../src/binary_dict.h"
#include <iostream>

int main() {
    akaza::BinaryDict dic;
    dic.load("akaza_data/data/system_dict.trie");
    std::cout << "FIND_KANJIS" << std::endl;
    {
        auto kanjis = dic.find_kanjis("あいう");
        for (auto & kanji: kanjis) {
            std::cout << kanji << std::endl;
        }
    }
    std::cout << "PREFIXES" << std::endl;
    {
        auto kanjis = dic.prefixes("あいうえお");
        for (auto & kanji: kanjis) {
            std::cout << kanji << std::endl;
        }
    }
}
