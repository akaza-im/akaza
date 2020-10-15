#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"

int main() {
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
    std::cout << std::get<1>(system_unigram_lm->find_unigram(L"私/わたし")) << std::endl;

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

    auto graphResolver = std::make_shared<akaza::GraphResolver>(
            user_language_model,
            system_unigram_lm,
            system_bigram_lm,
            normal_dicts,
            single_term_dicts
    );

    std::map<std::string, std::string> additional = {};
    auto romkan = std::make_shared<akaza::RomkanConverter>(additional);

    akaza::Akaza akaza = akaza::Akaza(graphResolver, romkan);
    std::vector<akaza::Slice> slices;
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> got = akaza.convert("watasinonamaehanakanodesu.",
                                                                               std::nullopt);

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv; // TODO remove
    std::wstring g;
    for (const auto &nodes: got) {
        g += nodes[0]->get_word();
        std::cout << "# ";
        for (const auto &node: nodes) {
            std::cout << cnv.to_bytes(node->get_key()) << "\t";
        }
        std::cout << std::endl;
    }

    ok(g == L"私の名前は中野です。");
    ok(got.size() == 7);

    done_testing();
}
