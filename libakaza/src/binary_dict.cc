#include "../include/akaza.h"
#include "nanoutf8.h"
#include "debug_log.h"

void akaza::BinaryDict::load(const std::string& dict_path) {
    dict_trie.load(dict_path.c_str());
    D(std::cout << "Loading BinaryDict: " << dict_path << ": " << dict_trie.num_keys()
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
}
