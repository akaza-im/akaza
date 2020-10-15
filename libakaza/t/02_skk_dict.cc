#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../include/skkdict.h"
#include <iostream>
#include <filesystem>

int main() {
    std::filesystem::path path(__FILE__);
    std::string spath = path.parent_path().concat("/data/SKK-JISYO.akaza").string();
    auto got = akaza::parse_skkdict(spath);

    auto ari = std::get<0>(got);
    auto nasi = std::get<1>(got);

    ok(ari.empty());
    ok(nasi.size() == 2);

    {
        std::vector<std::string> bukawa = nasi["ぶかわ"];
        std::vector<std::string> expected = {"武川"};
        ok(bukawa == expected);
    }
    {
        std::vector<std::string> bukawa = nasi["とくひろ"];
        std::vector<std::string> expected = {"徳宏","徳大","徳寛","督弘"};
        ok(bukawa == expected);
    }

    done_testing();
}
