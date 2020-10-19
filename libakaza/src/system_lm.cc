#include "debug_log.h"
#include "../include/system_lm.h"
#include <locale>
#include <codecvt>
#include <cstring>
#include <iostream>

void akaza::SystemUnigramLM::load(const char *path) {
    trie_.load(path);
    D(std::cout << "Loading SystemUnigramLM " << path << " size: " << trie_.size()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}

std::tuple<int32_t, float> akaza::SystemUnigramLM::find_unigram(const std::wstring &word) const {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;

    std::string query(cnv.to_bytes(word));
    query += "\xff"; // add marker

    marisa::Agent agent;
    agent.set_query(query.c_str(), query.size());

    while (trie_.predictive_search(agent)) {
        // dump_string(std::string(agent.key().ptr(), agent.key().length()));
        // std::cout << "HIT! " << std::endl;

        const char *p = agent.key().ptr() + query.size();
        float score = 0;
        std::memcpy(&score, p, sizeof(float));
        return std::tuple<int32_t, float>(int32_t(agent.key().id()), score);
    }
    return std::tuple<int32_t, float>(UNKNOWN_WORD_ID, 0);
}

void akaza::SystemUnigramLM::dump() {
    marisa::Agent agent;
    agent.set_query("");
    while (trie_.predictive_search(agent)) {
        std::string str(agent.key().ptr(), agent.key().length());
        size_t pos = str.find_first_of('\xff');
        std::string key = str.substr(0, pos);
        std::string scorestr = str.substr(pos + 1);
        float score = 0;
        std::memcpy(&score, scorestr.c_str(), sizeof(float));
        std::cout << key << "\t" << score << std::endl;
    }
}

void akaza::SystemBigramLM::load(const char *path) {
    trie_.load(path);
    D(std::cout << "Loading SystemBigramLM " << path << " size: " << trie_.size()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}

float akaza::SystemBigramLM::find_bigram(int32_t word_id1, int32_t word_id2) const {
    uint32_t uword_id1 = word_id1;
    uint32_t uword_id2 = word_id2;
    uint8_t idbuf[4];
    std::string query;
    std::memcpy(idbuf, &uword_id1, sizeof(word_id1));
    query += std::string(idbuf, idbuf + 3);
    std::memcpy(idbuf, &uword_id2, sizeof(word_id2));
    query += std::string(idbuf, idbuf + 3);

    marisa::Agent agent;
    agent.set_query(query.c_str(), query.size());

    while (trie_.predictive_search(agent)) {
        const char *p = agent.key().ptr() + query.size();
        float score;
        std::memcpy(&score, p, sizeof(float));
        return score;
    }
    return 0;
}

void akaza::SystemUnigramLMBuilder::add(const std::string &word, float score) {
    char buf[sizeof(float)];
    memcpy(buf, &score, sizeof(float));
    std::string key(word + "\xff" + std::string(buf, sizeof(float)));
    keyset_.push_back(key.c_str(), key.size());
}

void akaza::SystemBigramLMBuilder::add(int32_t word_id1, int32_t word_id2, float score) {
    // ここで処理する。
    std::string keybuf;
    uint8_t idbuf[sizeof(int32_t)];
    char scorebuf[sizeof(float)];

    // packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608
    std::memcpy(idbuf, &word_id1, sizeof(word_id1));
    keybuf += std::string(idbuf, idbuf + 3);
    std::memcpy(idbuf, &word_id2, sizeof(word_id2));
    keybuf += std::string(idbuf, idbuf + 3);

    // packed float  # score: 4 bytes
    std::memcpy(scorebuf, &score, sizeof(score));
    keybuf += std::string(scorebuf, scorebuf + 4);

    keyset_.push_back(keybuf.c_str(), keybuf.size());
}
