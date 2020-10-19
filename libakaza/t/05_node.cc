#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"
#include "tmpfile.h"

static void test_surface() {
    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    auto lisp = akaza::tinylisp::TinyLisp();
    auto node = akaza::create_node(system_unigram_lm, 0, L"たしざんてすと", LR"((. "a" "b"))");
    ok(node->surface(lisp) == L"ab");
}

static void test_eq() {
    TmpFile unigramfile;
    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.save(unigramfile.get_name());

    auto system_unigram_lm = std::make_shared<akaza::SystemUnigramLM>();
    system_unigram_lm->load(unigramfile.get_name().c_str());

    auto a = akaza::create_node(system_unigram_lm, 0, L"あ", L"あ");
    auto b = akaza::create_node(system_unigram_lm, 0, L"い", L"い");

    ok(a == a);
    ok(b == b);
    ok(b != a);
}


int main() {
    test_surface();
    test_eq();

    done_testing();
}