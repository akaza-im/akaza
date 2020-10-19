#include <iostream>
#include <fstream>
#include <map>
#include <marisa.h>
#include <locale>
#include <codecvt>
#include <sstream>
#include <fstream>


#include "../include/akaza.h"

/**

# 1gram

    {word} # in utf-8
    \xff   # marker
    packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608
    packed float  # score: 4 bytes

# 2gram

    {word1 ID}    # 3 bytes
    {word2 ID}    # 3 bytes
    packed float  # score: 4 bytes

*/

void process_1gram(const std::string &srcpath, const std::string &dstpath) {
    std::ifstream ifs(srcpath, std::ifstream::in);

    std::string line;
    akaza::SystemUnigramLMBuilder builder;
    int i = 0;
    while (std::getline(ifs, line)) {
        std::stringstream ss(line);

        std::string word;
        float score;
        ss >> word;
        ss >> score;

        // std::cout << word << "--" << score << std::endl;

        // ここで処理する。
        builder.add(word, score);

        if (++i >= 8388608) {
            // 3 byte に ID が収まる必要がある
            std::cerr << "too much words." << std::endl;
            exit(1);
        }
    }

    builder.save(dstpath);
}

void process_2gram(const akaza::SystemUnigramLM &unigram, const std::string &srcpath, const std::string &dstpath) {
    std::wifstream ifs(srcpath, std::ifstream::in);
    ifs.imbue(std::locale(std::locale(), new std::codecvt_utf8<wchar_t>));

    akaza::SystemBigramLMBuilder builder;
    std::wstring line;
    while (std::getline(ifs, line)) {
        std::wstringstream ss(line);
        std::wstring word1;
        std::wstring word2;
        float score;
        ss >> word1;
        ss >> word2;
        ss >> score;

        std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv; // TODO remove

//        std::cout << word1 << " --- " << word2 << " --- " << score << std::endl;
        int word_id1 = std::get<0>(unigram.find_unigram(word1));
        int word_id2 = std::get<0>(unigram.find_unigram(word2));

        builder.add(word_id1, word_id2, score);
    }

    builder.save(dstpath);
}

int main(int argc, char **argv) {
    // 1gram ファイルから読む。
    // 1gram の map<string, int> の ID mapping を作成する
    // 1gram データを書いていく。

    // "work/jawiki.merged-1gram.txt" "akaza_data/data/lm_v2_1gram.trie"
    // "work/jawiki.merged-2gram.txt" "akaza_data/data/lm_v2_2gram.trie"

    const char *unigram_src = argv[1];
    const char *unigram_dst = argv[2];
    const char *bigram_src = argv[3];
    const char *bigram_dst = argv[4];

    std::map<std::string, uint32_t> word2id;
    // "work/jawiki.merged-1gram.txt" -> "akaza_data/data/lm_v2_1gram.trie"
    std::cout << "Unigram: " << unigram_src << " -> " << unigram_dst << std::endl;

    process_1gram(unigram_src, unigram_dst);


    // 2gram ファイルから読む
    // 2gram ファイルを書いていく。
    std::cout << "Bigram: " << bigram_src << " -> " << bigram_dst << std::endl;
    akaza::SystemUnigramLM unigram;
    unigram.load(unigram_dst);
    process_2gram(unigram, bigram_src, bigram_dst);

    std::cout << "DONE" << std::endl;
    return 0;
}
