#include "../include//binary_dict.h"
#include <iostream>
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include <cstdlib>
#include <unistd.h>

int main() {
    char *dictfile = strdup("dict.XXXXXX");
    mkstemp(dictfile);

    // vector of "とくひろ" => "徳宏/徳大/徳寛/督弘"
    // void build(std::vector<std::tuple<std::string, std::string>> data) {
    {
        // building
        akaza::BinaryDict dic;
        std::vector<std::tuple<std::string, std::string>> list;
        list.emplace_back("あいう", "藍宇");
        dic.build(list);
        dic.save(dictfile);
    }

    {
        // saving
        akaza::BinaryDict dic;
        dic.load(dictfile);
        {
            auto kanjis = dic.find_kanjis("あいう");
            for (auto & kanji: kanjis) {
                note("%s", kanji.c_str());
            }
            ok(!kanjis.empty());
        }

        {
            std::vector<std::string> kanjis = dic.prefixes("あいうえお");
            for (auto & kanji: kanjis) {
                note("%s", kanji.c_str());
            }
            ok(kanjis.size() == 1);
        }
    }

    unlink(dictfile);
    free(dictfile);

    done_testing();
}
