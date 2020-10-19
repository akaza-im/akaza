#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "test_akaza.h"
#include <filesystem>

std::wstring convert_test(const std::wstring &wsrc, const std::wstring &expected) {
    auto akaza = build_akaza();
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza->convert(
            wsrc,
            std::nullopt);

    std::wstring retval;
    for (const auto &nodes: result) {
        retval += nodes[0]->get_word();
    }
    // note("RESULT: src=%s got=%s expected=%s", cnv.to_bytes(wsrc).c_str(), retval.c_str(), expected.c_str());

    std::wcout << "# RESULT: src=" << wsrc
               << " got=" << retval
               << " expected=" << expected << std::endl;

    ok(expected == retval);
    assert(expected == retval);
    return retval;
}

int main() {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    convert_test(L"けいやくのしゅたいとは", L"契約の主体とは");
    convert_test(L"tanosiijikan", L"楽しい時間");
    convert_test(L"たのしいじかん", L"楽しい時間");
    convert_test(L"zh", L"←");
    convert_test(L"それなwww", L"それなwww");
    convert_test(L"watasinonamaehanakanodesu.", L"私の名前は中野です。");
    convert_test(L"わたしのなまえはなかのです。", L"私の名前は中野です。");
    convert_test(L"わーど", L"ワード");
    convert_test(L"にほん", L"日本");
    convert_test(L"にっぽん", L"日本");
    convert_test(L"siinn", L"子音");
    convert_test(L"IME", L"IME");
    convert_test(L"ややこしい", L"ややこしい");
    convert_test(L"むずかしくない", L"難しく無い");
    convert_test(L"きぞん", L"既存");
    convert_test(L"のぞましい", L"望ましい");
    convert_test(L"こういう", L"こういう");
    convert_test(L"はやくち", L"早口");
    convert_test(L"しょうがっこう", L"小学校");
    convert_test(L"げすとだけ", L"ゲストだけ");
    convert_test(L"ぜんぶでてるやつ", L"全部でてるやつ");
    convert_test(L"えらべる", L"選べる");
    convert_test(L"わたしだよ", L"わたしだよ");
    convert_test(L"にほんごじょうほう", L"日本語情報");
    // convert_test(L"そうみたいですね", L"そうみたいですね");
    // convert_test(L"きめつのやいば", L"鬼滅の刃");
    convert_test(L"れいわ", L"令和");
    convert_test(L"ちいさい", L"小さい");
    done_testing();
}
