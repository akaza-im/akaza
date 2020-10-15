#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "test_akaza.h"
#include <filesystem>

std::wstring convert_test(const std::wstring &src, const std::wstring &expected) {
    auto akaza = build_akaza();
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza->convert(
            src,
            std::nullopt);

    std::wstring retval;
    for (const auto &nodes: result) {
        retval += nodes[0]->get_word();
    }
    std::wcout << "# RESULT: src=" << src << " got=" << retval
               << " expected=" << expected << std::endl;
    ok(expected == retval);
    assert(expected == retval);
    return retval;
}

int main() {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    convert_test(L"わたしのなまえはなかのです", L"私の名前は中野です");
    convert_test(L"わたしのなまえはなかのです。", L"私の名前は中野です。");
    done_testing();
}
