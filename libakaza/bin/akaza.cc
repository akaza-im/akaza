#include "../include/akaza.h"
#include <memory>
#include <unistd.h>
#include <string>

static void print_help(const char *name) {
    std::cout << "Usage: " << name << " words" << std::endl;
}

static std::string xdg_get_config_dir() {
    char *x = getenv("XDG_CONFIG_HOME");
    if (x) {
        return x;
    }
    char *home = getenv("HOME");
    if (home == nullptr) {
        throw std::runtime_error("Cannot get home directory");
    }
    return std::string(home) + "/.config";
}

static void show_unigram_cost(const std::shared_ptr<akaza::SystemUnigramLM> &systemUnigramLm,
                              const std::shared_ptr<akaza::UserLanguageModel> &userLanguageModel,
                              const std::wstring &key) {
    auto[word_id, cost] = systemUnigramLm->find_unigram(key);
    std::wcout << key << "\tsystem_word_id=" << word_id << " system_cost=" << cost;
    auto unicost = userLanguageModel->get_unigram_cost(key);
    if (unicost.has_value()) {
        std::wcout << " user_model=" << unicost.value();
    } else {
        std::wcout << " user_model=-";
    }
    std::wcout << std::endl;
}

static void show_bigram_cost(
        const std::shared_ptr<akaza::SystemUnigramLM> &systemUnigramLm,
        const std::shared_ptr<akaza::SystemBigramLM> &systemBigramLm,
        const std::shared_ptr<akaza::UserLanguageModel> &userLanguageModel,
        const std::wstring &key1,
        const std::wstring &key2) {
    auto[word_id1, cost1] = systemUnigramLm->find_unigram(key1);
    auto[word_id2, cost2] = systemUnigramLm->find_unigram(key2);
    std::wcout << key1 << "->" << key2 << "\t";
    if (word_id1 != akaza::UNKNOWN_WORD_ID && word_id2 != akaza::UNKNOWN_WORD_ID) {
        float cost = systemBigramLm->find_bigram(word_id1, word_id2);
        std::wcout << " system_bigram=" << cost;
    }
    auto bicost = userLanguageModel->get_bigram_cost(key1, key2);
    if (bicost.has_value()) {
        std::wcout << " user_model=" << bicost.value();
    } else {
        std::wcout << " user_model=-";
    }
    std::wcout << std::endl;
}

int main(int argc, char **argv) {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    const char *optstring = "v?";
    int c;
    bool verbose = false;

    while ((c = getopt(argc, argv, optstring)) != -1) {
        printf("opt=%c ", c);
        if (c == 'v') {
            verbose = true;
        } else {
            print_help(argv[0]);
            return -1; // error
        }
    }

    std::string configdir = xdg_get_config_dir();
    auto user_language_model = std::make_shared<akaza::UserLanguageModel>(
            configdir + "/ibus-akaza/user_language_model/unigram.txt",
            configdir + "/ibus-akaza/user_language_model/bigram.txt"
    );
    user_language_model->load_unigram();
    user_language_model->load_bigram();
    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load("/usr/share/akaza-data/lm_v2_1gram.trie");
    auto system_bigram_lm = std::make_shared<akaza::SystemBigramLM>();
    system_bigram_lm->load("/usr/share/akaza-data/lm_v2_2gram.trie");
    std::vector<std::shared_ptr<akaza::BinaryDict>> normal_dicts;
    auto system_dict = std::make_shared<akaza::BinaryDict>();
    system_dict->load("/usr/share/akaza-data/system_dict.trie");
    normal_dicts.push_back(system_dict);

    std::vector<std::shared_ptr<akaza::BinaryDict>> single_term_dicts;
    auto single_term = std::make_shared<akaza::BinaryDict>();
    single_term->load("/usr/share/akaza-data/single_term.trie");
    single_term_dicts.push_back(single_term);

    auto graphResolver = std::make_shared<akaza::GraphResolver>(
            user_language_model,
            system_unigram_lm,
            system_bigram_lm,
            normal_dicts,
            single_term_dicts
    );

    auto romkanConverter = akaza::build_romkan_converter({});

    akaza::Akaza akaza(graphResolver, romkanConverter);
    if (verbose) {
        std::cout << "# in verbose mode" << std::endl;
        auto kana = romkanConverter->to_hiragana(L"sitemo,");
        auto graph = graphResolver->graph_construct(kana, {});
        graphResolver->fill_cost(graph);
        graph.dump();
        auto result = graphResolver->find_nbest(graph);
        for (const auto &nodes: result) {
            std::wcout << nodes[0]->get_word() << "/";
        }
        std::wcout << std::endl;

        show_unigram_cost(system_unigram_lm, user_language_model, L"､/、");
        show_unigram_cost(system_unigram_lm, user_language_model, L"、/、");
        show_unigram_cost(system_unigram_lm, user_language_model, L"，/、");
        show_bigram_cost(system_unigram_lm, system_bigram_lm, user_language_model, L"しても/しても", L"，/、");
        show_bigram_cost(system_unigram_lm, system_bigram_lm, user_language_model, L"しても/しても", L"、/、");
    } else {
        std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza.convert(L"sitemo,",
                                                                                      std::nullopt);
        for (const auto &nodes: result) {
            std::wcout << nodes[0]->get_word() << "/";
        }
        std::wcout << std::endl;
    }
}

