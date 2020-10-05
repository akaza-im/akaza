#include "../src/binary_dict.h"
#include <iostream>
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

int main() {
    akaza::BinaryDict dic;
    dic.load("akaza_data/data/system_dict.trie");
    std::cout << "FIND_KANJIS" << std::endl;
    {
        auto kanjis = dic.find_kanjis("あいう");
        for (auto & kanji: kanjis) {
            std::cout << kanji << std::endl;
        }
        ok(kanjis.size() > 0);
    }
    std::cout << "PREFIXES" << std::endl;
    {
        auto kanjis = dic.prefixes("あいうえお");
        for (auto & kanji: kanjis) {
            std::cout << kanji << std::endl;
        }
    }
    done_testing();
}
