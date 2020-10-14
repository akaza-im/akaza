#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "test_akaza.h"
#include <filesystem>

std::string convert_test(const std::string &src, const std::string &expected) {
    auto akaza = build_akaza();
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza->convert(
            src,
            std::nullopt);

    std::string retval;
    for (const auto &nodes: result) {
        retval += nodes[0]->get_word();
    }
    note("RESULT: src=%s got=%s expected=%s", src.c_str(), retval.c_str(), expected.c_str());
    ok(expected == retval);
    assert(expected == retval);
    return retval;
}

int main() {
    convert_test("わたしのなまえはなかのです。", "私の名前は中野です。");
    // convert_test("わたしのなまえはなかのです", "私の名前は中野です");
    done_testing();
}
