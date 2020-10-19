#include "../include/akaza.h"
#include <memory>
#include <unistd.h>
#include <sstream>

static void print_help(const char *name) {
    std::cout << "Usage: " << name << " words" << std::endl;
}

static void run(akaza::Akaza &akaza, bool verbose) {
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result = akaza.convert(
            L"nagakunattekuruto,henkannninyojitunijikanngakakaruyouninattekuru.",
            std::nullopt);
    if (verbose) {
        for (const auto &nodes: result) {
            std::wcout << nodes[0]->get_word() << " ";
        }
        std::wcout << std::endl;
    }
}

int main(int argc, char **argv) {
    std::wostream::sync_with_stdio(false);
    std::wcout.imbue(std::locale("en_US.utf8"));

    const char *optstring = "n:v?";
    int c;
    int ntests = 100;
    bool verbose = false;

    while ((c = getopt(argc, argv, optstring)) != -1) {
        printf("opt=%c ", c);
        if (c == 'v') {
            verbose = true;
        } else if (c == 'n') {
            std::stringstream ss(optarg);
            ss >> ntests;
        } else {
            print_help(argv[0]);
            return -1; // error
        }
    }

    auto user_language_model = std::make_shared<akaza::UserLanguageModel>("/tmp/uni", "/tmp/bi");
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
    run(akaza, true);

    std::cout << "ntests=" << ntests << std::endl;
    std::chrono::steady_clock::time_point begin = std::chrono::steady_clock::now();
    for (int i = 0; i < ntests; i++) {
        run(akaza, false);
    }
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::cout << "Elapsed time = " << std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count()
              << "[ms]" << std::endl;
    std::cout << "Performance = "
              << float(std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count()) / float(ntests)
              << "[ms]" << std::endl;
}
