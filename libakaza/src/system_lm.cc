#include "../include/akaza.h"

#include "debug_log.h"

void akaza::SystemUnigramLM::load(const char *path) {
    trie.load(path);
    D(std::cout << "Loading SystemUnigramLM " << path << " size: " << trie.size()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}

void akaza::SystemBigramLM::load(const char *path) {
    trie.load(path);
    D(std::cout << "Loading SystemBigramLM " << path << " size: " << trie.size()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}
