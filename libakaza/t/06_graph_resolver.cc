#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"
#include "test_akaza.h"

// 「ひょいー」のような辞書に登録されていない単語に対して、カタカナ候補を提供すべき。
static void test_katakana_candidates() {
    auto graph_resolver = build_graph_resolver();
    auto graph = graph_resolver->graph_construct(
            L"ひょいー",
            std::make_optional<std::vector<akaza::Slice>>({
                                                                  akaza::Slice(
                                                                          0,
                                                                          4)
                                                          }));
    graph_resolver->fill_cost(graph);
    auto got = graph_resolver->find_nbest(graph);
    std::set<std::wstring> words;
    for (const auto &node: got[0]) {
        words.insert(node->get_word());
    }
    ok(words.count(L"ひょいー") == 1);
    ok(words.count(L"ヒョイー") == 1);
}

// 「すし」の変換結果。
static void test_sushi() {
    auto graph_resolver = build_graph_resolver();
    auto graph = graph_resolver->graph_construct(
            L"すし",
            std::make_optional<std::vector<akaza::Slice>>({
                                                                  akaza::Slice(
                                                                          0,
                                                                          2)
                                                          }));
    graph_resolver->fill_cost(graph);
    auto got = graph_resolver->find_nbest(graph);
    std::set<std::wstring> words;
    for (const auto &node: got[0]) {
        words.insert(node->get_word());
        // std::cout << node->get_word() << std::endl;
    }
    ok(words.count(L"🍣") == 1);
    ok(words.count(L"鮨") == 1);
}

int main() {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    TmpFile unigramPath;
    TmpFile bigramPath;

    std::shared_ptr<akaza::UserLanguageModel> user_language_model(
            new akaza::UserLanguageModel("a", "b"));

    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.add("私/わたし", -0.01);
    unibuilder.add("中野/なかの", -0.01);
    unibuilder.add("名前/なまえ", -0.01);
    unibuilder.save(unigramPath.get_name());

    std::shared_ptr<akaza::SystemUnigramLM> system_unigram_lm(
            new akaza::SystemUnigramLM()
    );
    system_unigram_lm->load(unigramPath.get_name().c_str());

    akaza::SystemBigramLMBuilder bibuilder;
    bibuilder.save(bigramPath.get_name());

    std::shared_ptr<akaza::SystemBigramLM> system_bigram_lm(
            new akaza::SystemBigramLM()
    );
    system_bigram_lm->load(bigramPath.get_name().c_str());

    std::vector<std::shared_ptr<akaza::BinaryDict>> normal_dicts;
    std::shared_ptr<akaza::BinaryDict> dict(new akaza::BinaryDict());
    std::vector<std::tuple<std::string, std::string>> ddd;
    ddd.emplace_back("わたし", "私/渡し");
    ddd.emplace_back("の", "の");
    ddd.emplace_back("なまえ", "名前");
    ddd.emplace_back("は", "は");
    ddd.emplace_back("なかの", "中野");
    ddd.emplace_back("です", "です");
    ddd.emplace_back("。", "。");
    dict->build(ddd);
    normal_dicts.push_back(dict);

    std::vector<std::shared_ptr<akaza::BinaryDict>> single_term_dicts;

    akaza::GraphResolver graphResolver(
            user_language_model,
            system_unigram_lm,
            system_bigram_lm,
            normal_dicts,
            single_term_dicts
    );
    akaza::Graph graph = graphResolver.graph_construct(L"わたしのなまえはなかのです。", std::nullopt);

    {
        int desu = 0;
        int maru = 0;
        for (const auto &node: graph.get_items()) {
            if (node->get_key() == L"です/です") {
                desu++;
            }
            if (node->get_key() == L"。/。") {
                maru++;
            }
        }
        ok(desu == 1);
        ok(maru == 1);
    }

    graph.dump();

    graphResolver.fill_cost(graph);

    graph.dump();

    std::vector<std::vector<std::shared_ptr<akaza::Node>>> got = graphResolver.find_nbest(graph);
    std::wstring g;
    for (const auto &nodes: got) {
        g += nodes[0]->get_word();
        for (const auto &node: nodes) {
            std::wcout << node->get_key() << " ";
        }
        std::cout << std::endl;
    }
    ok(g == L"私の名前は中野です。");
    ok(!got.empty());

    test_katakana_candidates();
    test_sushi();

    done_testing();
}

