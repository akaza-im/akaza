#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../include/skkdict.h"
#include <iostream>

int main() {
    auto got = akaza::parse_skkdict("t/data/SKK-JISYO.akaza");

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
