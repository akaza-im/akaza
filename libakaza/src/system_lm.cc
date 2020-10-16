#include "../include/akaza.h"

#include "debug_log.h"
#include "../include/system_lm.h"


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
