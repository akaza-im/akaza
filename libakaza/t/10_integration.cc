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

    std::wstring retval;
    for (const auto &nodes: result) {
        retval += nodes[0]->get_word();
    }
    note("RESULT: src=%s got=%s expected=%s", src.c_str(), retval.c_str(), expected.c_str());
     std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv; // TODO remove

    ok(expected == cnv.to_bytes(retval));
    assert(expected == cnv.to_bytes(retval));
    return cnv.to_bytes(retval);
}

int main() {
    convert_test("tanosiijikan", "楽しい時間");
    convert_test("たのしいじかん", "楽しい時間");
    convert_test("zh", "←");
    convert_test("それなwww", "それなwww");
    convert_test("watasinonamaehanakanodesu.", "私の名前は中野です。");
    convert_test("わたしのなまえはなかのです。", "私の名前は中野です。");
    convert_test("わーど", "ワード");
    convert_test("にほん", "日本");
    convert_test("にっぽん", "日本");
    convert_test("siinn", "子音");
    convert_test("IME", "IME");
    convert_test("ややこしい", "ややこしい");
    convert_test("むずかしくない", "難しく無い");
    convert_test("きぞん", "既存");
    convert_test("のぞましい", "望ましい");
    convert_test("こういう", "こういう");
    convert_test("はやくち", "早口");
    convert_test("しょうがっこう", "小学校");
    convert_test("げすとだけ", "ゲストだけ");
    convert_test("ぜんぶでてるやつ", "全部でてるやつ");
    convert_test("えらべる", "選べる");
    convert_test("わたしだよ", "私だよ");
    convert_test("にほんごじょうほう", "日本語情報");
    // convert_test("そうみたいですね", "そうみたいですね");
    // convert_test("きめつのやいば", "鬼滅の刃");
    // convert_test("れいわ", "令和");
    done_testing();
}
