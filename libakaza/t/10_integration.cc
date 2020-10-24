#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "test_akaza.h"
#include <filesystem>

static std::wstring convert_test(std::unique_ptr<akaza::Akaza> &akaza,
                                 const std::wstring &wsrc, const std::wstring &expected) {
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza->convert(
            wsrc,
            std::nullopt);

    std::wstring retval;
    for (const auto &nodes: result) {
        retval += nodes[0]->get_word();
    }

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
    std::unique_ptr<akaza::Akaza> akaza = build_akaza();

    convert_test(akaza, L"http://mixi.jp", L"http://mixi.jp");
    convert_test(akaza, L"https://mixi.jp", L"https://mixi.jp");
    convert_test(akaza, L"nisitemo,", L"にしても、");
    convert_test(akaza, L"けいやくないようをめいかくにするいぎ", L"契約内容を明確にする意義");
    convert_test(akaza, L"ろうどうしゃさいがいほしょうほけんほう", L"労働者災害補償保険法");
    convert_test(akaza, L"けいやくのしゅたいとは", L"契約の主体とは");
    convert_test(akaza, L"tanosiijikan", L"楽しい時間");
    convert_test(akaza, L"たのしいじかん", L"楽しい時間");
    convert_test(akaza, L"zh", L"←");
    convert_test(akaza, L"それなwww", L"それなwww");
    convert_test(akaza, L"watasinonamaehanakanodesu.", L"私の名前は中野です。");
    convert_test(akaza, L"わたしのなまえはなかのです。", L"私の名前は中野です。");
    convert_test(akaza, L"わーど", L"ワード");
    convert_test(akaza, L"にほん", L"日本");
    convert_test(akaza, L"にっぽん", L"日本");
    convert_test(akaza, L"siinn", L"子音");
    convert_test(akaza, L"IME", L"IME");
    convert_test(akaza, L"ややこしい", L"ややこしい");
    convert_test(akaza, L"むずかしくない", L"難しく無い");
    convert_test(akaza, L"きぞん", L"既存");
    convert_test(akaza, L"のぞましい", L"望ましい");
    convert_test(akaza, L"こういう", L"こういう");
    convert_test(akaza, L"はやくち", L"早口");
    convert_test(akaza, L"しょうがっこう", L"小学校");
    convert_test(akaza, L"げすとだけ", L"ゲストだけ");
    convert_test(akaza, L"ぜんぶでてるやつ", L"全部でてるやつ");
    convert_test(akaza, L"えらべる", L"選べる");
    convert_test(akaza, L"わたしだよ", L"わたしだよ");
    convert_test(akaza, L"にほんごじょうほう", L"日本語情報");
    // convert_test(akaza, L"そうみたいですね", L"そうみたいですね");
    // convert_test(akaza, L"きめつのやいば", L"鬼滅の刃");
    convert_test(akaza, L"れいわ", L"令和");
    convert_test(akaza, L"ちいさい", L"小さい");
    done_testing();
}
