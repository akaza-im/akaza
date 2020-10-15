#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include <cstdlib>
#include <unistd.h>

#include "tmpfile.h"


void test_read() {
    TmpFile unigram_path;
    TmpFile bigram_path;

    akaza::UserLanguageModel d(unigram_path.get_name(), bigram_path.get_name());

    d.add_entry({akaza::Node(0, L"たんご", L"単語")});
    d.add_entry({akaza::Node(0, L"たんご", L"単語")});
    d.add_entry({akaza::Node(0, L"じゅくご", L"熟語")});

    ok(d.get_unigram_cost(L"単語/たんご") > d.get_unigram_cost(L"熟語/じゅくご"));
}

/*
 def test_read3():
    tmpdir = TemporaryDirectory()
    user_language_model = UserLanguageModel(tmpdir.name + "/foobar.dict")
    user_language_model.add_entry([
        Node(start_pos=0, word='ヒョイー', yomi='ひょいー'),
    ])

    assert user_language_model.unigram == {'ヒョイー/ひょいー': 1}
    assert user_language_model.has_unigram_cost_by_yomi('ひょいー')
 */
void test_read3() {
    char *unigram_path = strdup("dict.XXXXXX");
    mkstemp(unigram_path);
    char *bigram_path = strdup("uni.XXXXXX");
    mkstemp(bigram_path);

    akaza::UserLanguageModel d(unigram_path, bigram_path);
    d.add_entry({akaza::Node(0, L"ひょいー", L"ヒョイー")});
    ok(d.has_unigram_cost_by_yomi(L"ひょいー") == true);

    unlink(unigram_path);
    free(unigram_path);
    unlink(bigram_path);
    free(bigram_path);
}

/*
def test_read2():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='それは', yomi='それは'),
        Node(start_pos=3, word='私', yomi='わたし'),
        Node(start_pos=4, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='です', yomi='です'),
    ])

    assert d.unigram == {'それは/それは': 1, 'だよ/だよ': 2, '私/わたし': 3, 'です/です': 1}
    assert d.unigram_C == 4
    assert d.unigram_V == 7

    assert d.bigram == {'それは/それは\t私/わたし': 1, '私/わたし\tだよ/だよ': 2, '私/わたし\tです/です': 1}
    assert d.bigram_C == 3
    assert d.bigram_V == 4

 */
void test_read2() {
    char *unigram_path = strdup("dict.XXXXXX");
    mkstemp(unigram_path);
    char *bigram_path = strdup("uni.XXXXXX");
    mkstemp(bigram_path);

    akaza::UserLanguageModel d(unigram_path, bigram_path);

    d.add_entry({
                        akaza::Node(0, L"わたし", L"私"),
                        akaza::Node(0, L"だよ", L"だよ")
                });
    d.add_entry({
                        akaza::Node(0, L"それは", L"それは"),
                        akaza::Node(0, L"わたし", L"私"),
                        akaza::Node(0, L"だよ", L"だよ")
                });
    d.add_entry({
                        akaza::Node(0, L"わたし", L"私"),
                        akaza::Node(0, L"です", L"です")
                });


    unlink(unigram_path);
    free(unigram_path);
    unlink(bigram_path);
    free(bigram_path);
}

int main() {
    test_read();
    test_read2();
    test_read3();

    done_testing();
}
