#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../include/system_lm.h"
#include <cstdlib>
#include <unistd.h>

int main() {
    char *unigramfile = strdup("unigram.XXXXXX");
    mkstemp(unigramfile);
    char *bigramfile = strdup("unigram.XXXXXX");
    mkstemp(bigramfile);

    akaza::SystemUnigramLMBuilder unibuilder;
    unibuilder.add("私/わたし", -0.3);
    unibuilder.add("は/は", -0.4);
    unibuilder.add("じゃ/じゃ", -0.9);
    unibuilder.save(unigramfile);

    akaza::SystemUnigramLM uni;
    uni.load(unigramfile);
    int id_watashi = std::get<0>(uni.find_unigram("私/わたし"));
    int id_ha = std::get<0>(uni.find_unigram("は/は"));
    int id_ja = std::get<0>(uni.find_unigram("じゃ/じゃ"));
    ok(std::abs(std::get<1>(uni.find_unigram("私/わたし")) - -0.3) < 0.000001);
    ok(std::abs(std::get<1>(uni.find_unigram("は/は")) - -0.4) < 0.000001);

    akaza::SystemBigramLMBuilder bibuilder;
    bibuilder.add(id_watashi, id_ha, -0.3);
    bibuilder.add(id_watashi, id_ja, -0.4);
    bibuilder.save(bigramfile);

    akaza::SystemBigramLM bi;
    bi.load(bigramfile);
    ok(std::abs(bi.find_bigram(id_watashi, id_ha) - -0.3) < 0.000001);
    ok(std::abs(bi.find_bigram(id_watashi, id_ja) - -0.4) < 0.000001);

    unlink(unigramfile);
    unlink(bigramfile);
    free(unigramfile);
    free(bigramfile);

    done_testing();
}
