#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"

static void test_remove_last_char() {
    auto romkan = akaza::build_romkan_converter({});

    std::vector<std::tuple<std::wstring, std::wstring>> cases = {
            {L"aka",    L"a"},
            {L"sona",   L"so"},
            {L"son",    L"so"},
            {L"sonn",   L"so"},
            {L"sonnna", L"sonn"},
            {L"sozh",   L"so"},
    };

    for (const auto &[src, expected]: cases) {
        auto got = romkan->remove_last_char(src);
        note("REMOVE_LAST_CHAR: %s -> %s", src.c_str(), got.c_str());
        ok(got == expected);
    }
}

static void test_to_hiragana() {
    auto romkan = akaza::build_romkan_converter({});

    std::vector<std::tuple<std::wstring, std::wstring>> cases = {
            {L"a",         L"あ"},
            {L"ba",        L"ば"},
            {L"hi",        L"ひ"},
            {L"wahaha",    L"わはは"},
            {L"thi",       L"てぃ"},
            {L"better",    L"べってr"},
            {L"[",         L"「"},
            {L"]",         L"」"},
            {L"wo",        L"を"},
            {L"du",        L"づ"},
            {L"we",        L"うぇ"},
            {L"di",        L"ぢ"},
            {L"fu",        L"ふ"},
            {L"ti",        L"ち"},
            {L"wi",        L"うぃ"},
            {L"we",        L"うぇ"},
            {L"wo",        L"を"},
            {L"z,",        L"‥"},
            {L"z.",        L"…"},
            {L"z/",        L"・"},
            {L"z[",        L"『"},
            {L"z]",        L"』"},
            {L"du",        L"づ"},
            {L"di",        L"ぢ"},
            {L"fu",        L"ふ"},
            {L"ti",        L"ち"},
            {L"wi",        L"うぃ"},
            {L"we",        L"うぇ"},
            {L"wo",        L"を"},
            {L"sorenawww", L"それなwww"},
            {L"komitthi",  L"こみってぃ"},
            {L"ddha",      L"っでゃ"},
            {L"zzye",       L"っじぇ"},};

    for (const auto &[src, expected]: cases) {
        auto got = romkan->to_hiragana(src);
        std::wcout << "# HIRAGANA: " << src << " " << got << std::endl;
        ok(got == expected);
    }
}

int main() {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    test_remove_last_char();
    test_to_hiragana();

    auto romkan = akaza::build_romkan_converter({});

    auto got = romkan->to_hiragana(L"akasatana");
    std::wcout << "# " << got << std::endl;
    ok(got == L"あかさたな");

    done_testing();
}
