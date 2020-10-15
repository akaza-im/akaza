#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"

/*
@pytest.mark.parametrize('src, expected', [
    ('aka', 'a'),
    ('sona', 'so'),
    ('son', 'so'),
    ('sonn', 'so'),
    ('sonnna', 'sonn'),
    ('sozh', 'so'),
])
def test_remove_last_char(src, expected):
    romkan = RomkanConverter()
    assert romkan.remove_last_char(src) == expected
 */
static void test_remove_last_char() {
    std::map<std::string, std::string> additional = {
    };
    auto romkan = akaza::RomkanConverter(additional);

    std::vector<std::tuple<std::wstring, std::wstring>> cases = {
            {L"aka",    L"a"},
            {L"sona",   L"so"},
            {L"son",    L"so"},
            {L"sonn",   L"so"},
            {L"sonnna", L"sonn"},
            {L"sozh",   L"so"},
    };

    for (const auto &[src, expected]: cases) {
        auto got = romkan.remove_last_char(src);
        note("REMOVE_LAST_CHAR: %s -> %s", src.c_str(), got.c_str());
        ok(got == expected);
    }
}

/*
@pytest.mark.parametrize('src, expected', [
])
def test_bar(src, expected):
    romkan = RomkanConverter()
    assert romkan.to_hiragana(src) == expected
 */
static void test_to_hiragana() {
    std::map<std::string, std::string> additional = {
    };
    auto romkan = akaza::RomkanConverter(additional);

    std::vector<std::tuple<std::string, std::string>> cases = {
            {"a",      "あ"},
            {"ba",     "ば"},
            {"hi",     "ひ"},
            {"wahaha", "わはは"},
            {"thi",    "てぃ"},
            {"better", "べってr"},
            {"[",      "「"},
            {"]",      "」"},
            {"wo",     "を"},
            {"du",     "づ"},
            {"we",     "うぇ"},
            {"di",     "ぢ"},
            {"fu",     "ふ"},
            {"ti",     "ち"},
            {"wi",     "うぃ"},
            {"we",     "うぇ"},
            {"wo",     "を"},
            {"z,",     "‥"},
            {"z.",     "…"},
            {"z/",     "・"},
            {"z[",     "『"},
            {"z]",     "』"},
            {"du",     "づ"},
            {"di",     "ぢ"},
            {"fu",     "ふ"},
            {"ti",     "ち"},
            {"wi",     "うぃ"},
            {"we",     "うぇ"},
            {"wo",     "を"},
            {"sorenawww",     "それなwww"},
    };

    for (const auto &[src, expected]: cases) {
        auto got = romkan.to_hiragana(src);
        note("HIRAGANA: %s -> %s", src.c_str(), got.c_str());
        ok(got == expected);
    }
}

int main() {
    test_remove_last_char();
    test_to_hiragana();

    std::map<std::string, std::string> additional = {
    };
    auto romkan = akaza::RomkanConverter(additional);

    auto got = romkan.to_hiragana("akasatana");
    note("%s", got.c_str());
    ok(got == "あかさたな");

    done_testing();
}
