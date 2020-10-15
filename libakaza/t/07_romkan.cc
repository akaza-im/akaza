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

    std::vector<std::tuple<std::wstring, std::wstring>> cases = {
            {L"a",      L"あ"},
            {L"ba",     L"ば"},
            {L"hi",     L"ひ"},
            {L"wahaha", L"わはは"},
            {L"thi",    L"てぃ"},
            {L"better", L"べってr"},
            {L"[",      L"「"},
            {L"]",      L"」"},
            {L"wo",     L"を"},
            {L"du",     L"づ"},
            {L"we",     L"うぇ"},
            {L"di",     L"ぢ"},
            {L"fu",     L"ふ"},
            {L"ti",     L"ち"},
            {L"wi",     L"うぃ"},
            {L"we",     L"うぇ"},
            {L"wo",     L"を"},
            {L"z,",     L"‥"},
            {L"z.",     L"…"},
            {L"z/",     L"・"},
            {L"z[",     L"『"},
            {L"z]",     L"』"},
            {L"du",     L"づ"},
            {L"di",     L"ぢ"},
            {L"fu",     L"ふ"},
            {L"ti",     L"ち"},
            {L"wi",     L"うぃ"},
            {L"we",     L"うぇ"},
            {L"wo",     L"を"},
            {L"sorenawww",     L"それなwww"},
    };

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    for (const auto &[src, expected]: cases) {
        auto got = romkan.to_hiragana(cnv.to_bytes(src));
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
    ok(got == L"あかさたな");

    done_testing();
}
