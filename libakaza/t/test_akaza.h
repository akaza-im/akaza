#pragma once

#include <memory>
#include <filesystem>

#include "../include/akaza.h"

static std::string akaza_path(const char *s) {
    std::filesystem::path p(__FILE__);
    std::string retval = p.parent_path().parent_path().parent_path()
            .concat("/akaza-data/data/").concat(s).string();
    note("Loading '%s'", retval.c_str());
    return retval;
}

static std::shared_ptr<akaza::GraphResolver> build_graph_resolver() {
    auto user_language_model = std::make_shared<akaza::UserLanguageModel>("/tmp/uni", "/tmp/bi");
    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(akaza_path("lm_v2_1gram.trie").c_str());
    auto system_bigram_lm = std::make_shared<akaza::SystemBigramLM>();
    system_bigram_lm->load(akaza_path("lm_v2_2gram.trie").c_str());
    std::vector<std::shared_ptr<akaza::BinaryDict>> normal_dicts;
    auto system_dict = std::make_shared<akaza::BinaryDict>();
    system_dict->load(akaza_path("system_dict.trie"));
    normal_dicts.push_back(system_dict);

    std::vector<std::shared_ptr<akaza::BinaryDict>> single_term_dicts;
    auto single_term = std::make_shared<akaza::BinaryDict>();
    single_term->load(akaza_path("single_term.trie"));
    single_term_dicts.push_back(single_term);

    std::shared_ptr<akaza::GraphResolver> graphResolver = std::make_shared<akaza::GraphResolver>(
            user_language_model,
            system_unigram_lm,
            system_bigram_lm,
            normal_dicts,
            single_term_dicts
    );
    return graphResolver;
}

static std::unique_ptr<akaza::Akaza> build_akaza() {
    auto graph_resolver = build_graph_resolver();
    auto romkanConverter = akaza::build_romkan_converter({});

    return std::make_unique<akaza::Akaza>(graph_resolver, romkanConverter);
}
