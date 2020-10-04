#define CATCH_CONFIG_MAIN  // This tells Catch to provide a main() - only do this in one cpp file
#include "catch.hpp"

#include "../src/skkdict.h"
#include <iostream>

TEST_CASE( "Parse SKK Dict", "[factorial]" ) {
    auto got = akaza::parse_skkdict("dict/SKK-JISYO.akaza");

    auto ari = std::get<0>(got);
    auto nasi = std::get<1>(got);

    REQUIRE(ari.size() == 0);
    REQUIRE(nasi.size() == 2);

    {
        std::vector<std::string> bukawa = nasi["ぶかわ"];
        std::vector<std::string> expected = {"武川"};
        REQUIRE(bukawa == expected);
    }
    {
        std::vector<std::string> bukawa = nasi["とくひろ"];
        std::vector<std::string> expected = {"徳宏","徳大","徳寛","督弘"};
        REQUIRE(bukawa == expected);
    }
}
