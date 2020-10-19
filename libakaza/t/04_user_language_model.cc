#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include <cstdlib>
#include <unistd.h>

#include "tmpfile.h"


void test_read() {
    TmpFile unigram_path;
    TmpFile bigram_path;

    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    akaza::UserLanguageModel d(unigram_path.get_name(), bigram_path.get_name());

    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"たんご", L"単語")});
    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"たんご", L"単語")});
    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"じゅくご", L"熟語")});

    ok(d.get_unigram_cost(L"単語/たんご") > d.get_unigram_cost(L"熟語/じゅくご"));
}

void test_read3() {
    char *unigram_path = strdup("dict.XXXXXX");
    mkstemp(unigram_path);
    char *bigram_path = strdup("uni.XXXXXX");
    mkstemp(bigram_path);

    akaza::UserLanguageModel d(unigram_path, bigram_path);

    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"ひょいー", L"ヒョイー")});
    ok(d.has_unigram_cost_by_yomi(L"ひょいー") == true);

    unlink(unigram_path);
    free(unigram_path);
    unlink(bigram_path);
    free(bigram_path);
}

void test_read2() {
    char *unigram_path = strdup("dict.XXXXXX");
    mkstemp(unigram_path);
    char *bigram_path = strdup("uni.XXXXXX");
    mkstemp(bigram_path);

    akaza::UserLanguageModel d(unigram_path, bigram_path);

    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    d.add_entry({
                        *akaza::create_node(system_unigram_lm, 0, L"わたし", L"私"),
                        *akaza::create_node(system_unigram_lm, 0, L"だよ", L"だよ")
                });
    d.add_entry({
                        *akaza::create_node(system_unigram_lm, 0, L"それは", L"それは"),
                        *akaza::create_node(system_unigram_lm, 0, L"わたし", L"私"),
                        *akaza::create_node(system_unigram_lm, 0, L"だよ", L"だよ")
                });
    d.add_entry({
                        *akaza::create_node(system_unigram_lm, 0, L"わたし", L"私"),
                        *akaza::create_node(system_unigram_lm, 0, L"です", L"です")
                });


    unlink(unigram_path);
    free(unigram_path);
    unlink(bigram_path);
    free(bigram_path);
}

void test_save() {
    TmpFile unigram_path;
    TmpFile bigram_path;

    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    akaza::UserLanguageModel d(unigram_path.get_name(), bigram_path.get_name());

    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"たんご", L"単語")});
    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"たんご", L"単語")});
    d.add_entry({*akaza::create_node(system_unigram_lm, 0, L"じゅくご", L"熟語")});
    d.save();

    ok(std::filesystem::file_size(unigram_path.get_name()) > 0);
}

int main() {
    test_read();
    test_read2();
    test_read3();
    test_save();

    done_testing();
}
