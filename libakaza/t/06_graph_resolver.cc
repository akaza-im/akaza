#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"

int main() {
    /*
     * const std::shared_ptr<UserLanguageModel> &user_language_model,
                      const std::shared_ptr<SystemUnigramLM> &system_unigram_lm,
                      const std::shared_ptr<SystemBigramLM> &system_bigram_lm,
                      std::vector<std::shared_ptr<BinaryDict>> normal_dicts,
                      std::vector<std::shared_ptr<BinaryDict>> single_term_dicts
     */
    TmpFile unigramPath;
    TmpFile bigramPath;

    std::shared_ptr<akaza::UserLanguageModel> user_language_model(
            new akaza::UserLanguageModel("a", "b"));

    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.add("私/わたし", -0.01);
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
    akaza::Graph graph = graphResolver.graph_construct("わたしのなまえはなかのです。", std::nullopt);

    {
        int desu = 0;
        int maru = 0;
        for (const auto &node: graph.get_items()) {
            if (node->get_key() == "です/です") {
                desu++;
            }
            if (node->get_key() == "。/。") {
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
    std::string g;
    for (const auto &nodes: got) {
        g += nodes[0]->get_word();
        for (const auto &node: nodes) {
            std::cout << node->get_key() << " ";
        }
        std::cout << std::endl;
    }
    note("%s", g.c_str());
    ok(g == "私の名前は中野です。");
    ok(!got.empty());

    done_testing();
}
