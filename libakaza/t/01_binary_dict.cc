#include "../include//binary_dict.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

int main() {
    // vector of "とくひろ" => "徳宏/徳大/徳寛/督弘"
    {
        // saving
        akaza::BinaryDict dic;
        {
            std::vector<std::tuple<std::string, std::string>> list;
            list.emplace_back("あいう", "藍宇");
            dic.build(list);
        }
        {
            auto kanjis = dic.find_kanjis(L"あいう");
            for (auto &kanji: kanjis) {
                note("%s", kanji.c_str());
            }
            ok(!kanjis.empty());
        }
    }

    done_testing();
}
