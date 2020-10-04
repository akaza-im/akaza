#include <iostream>
#include <fstream>
#include <map>
#include <cstring>
#include <marisa.h>

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

void process_1gram(std::map<std::string, uint32_t> &word2id) {
    std::ifstream ifs ("work/jawiki.merged-1gram.txt", std::ifstream::in);
    std::string word;
    marisa::Keyset keyset;

    uint32_t id=1;
    float score;
    uint8_t idbuf[4];
    uint8_t scorebuf[sizeof(float)];
    while (!ifs.eof()) {
        ifs >> word;
        ifs >> score;

        if (word2id.find(word) != word2id.end()) {
            break;
        }
        // std::cout << word << "--" << score << std::endl;
        word2id[word] = id;

        // ここで処理する。
        std::string keybuf(word);

        // marker
        keybuf += "\xff";

        // packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608
        std::memcpy(idbuf, &id, sizeof(id));
        keybuf += std::string(idbuf, idbuf+3);

        // packed float  # score: 4 bytes
        std::memcpy(scorebuf, &score, sizeof(score));
        keybuf += std::string(scorebuf, scorebuf+sizeof(score));

        if (word == "私/わたし" || word == "三/み") {
            std::cout << "WATASHI(1) " << keybuf << " " << ((id>>16) & 0xff)  << " SCORE=" <<score
                << " id=" << id << " HEX=";
            const char * q=keybuf.c_str();
            for (int i=0; i<keybuf.size(); i++) {
                std::cout << +((uint8_t)q[i]) << " ";
            }
            std::cout << std::endl;
        }

        if (word == "/") {
            std::cout << "SLASH(1) " << keybuf << " " << ((id>>16) & 0xff)  << " SCORE=" <<score
                << " id=" << id << " HEX=";
            const char * q=keybuf.c_str();
            for (int i=0; i<keybuf.size(); i++) {
                std::cout << +((uint8_t)q[i]) << " ";
            }
            std::cout << std::endl;
        }

        keyset.push_back(keybuf.c_str(), keybuf.size());

        id++;

        if (id>=8388608) {
            std::cerr << "too much words." << std::endl;
            exit(1);
        }
    }

    marisa::Trie trie;
    trie.build(keyset);
    trie.save("akaza_data/data/lm_v2_1gram.trie");
}

void process_2gram(std::map<std::string, uint32_t> &word2id) {
    std::ifstream ifs ("work/jawiki.merged-2gram.txt", std::ifstream::in);
    std::string word1;
    std::string word2;
    marisa::Keyset keyset;

    float score;
    char scorebuf[sizeof(float)];
    uint8_t idbuf[4];
    while (!ifs.eof()) {
        ifs >> word1;
        ifs >> word2;
        ifs >> score;

//        std::cout << word1 << " --- " << word2 << " --- " << score << std::endl;
        int id1 = word2id[word1];
        int id2 = word2id[word2];

        // ここで処理する。
        std::string keybuf;

        // packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608
        std::memcpy(idbuf, &id1, sizeof(id1));
        keybuf += std::string(idbuf, idbuf+3);
        std::memcpy(idbuf, &id2, sizeof(id2));
        keybuf += std::string(idbuf, idbuf+3);

        // packed float  # score: 4 bytes
        std::memcpy(scorebuf, &score, sizeof(score));
        keybuf += std::string(scorebuf, scorebuf+4);

        if (word1 == "私/わたし") {
            std::cout << "WATASI " << word1 << " " << word2 <<
                " SCORE=" <<score << " id1=" << id1 << " id2=" << id2 << " HEX=";
            const char * q=keybuf.c_str();
            for (int i=0; i<keybuf.size(); i++) {
                std::cout << +((uint8_t)q[i]) << " ";
            }
            std::cout << std::endl;
        }

        keyset.push_back(keybuf.c_str(), keybuf.size());
    }

    marisa::Trie trie;
    trie.build(keyset);
    trie.save("akaza_data/data/lm_v2_2gram.trie");
}

int main() {
    // 1gram ファイルから読む。
    // 1gram の map<string, int> の ID mapping を作成する
    // 1gram データを書いていく。

    std::map<std::string, uint32_t> word2id;
    process_1gram(word2id);

    std::cout << "BEFORE" << std::endl;

    // 2gram ファイルから読む
    // 2gram ファイルを書いていく。
    process_2gram(word2id);

    std::cout << "DONE" << std::endl;
    return 0;
}
