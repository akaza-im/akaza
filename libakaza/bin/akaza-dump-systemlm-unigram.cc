#include <iostream>
#include "../include/akaza.h"
#include <filesystem>

int main(int argc, char** argv) {
    if (argc != 2) {
        std::cout << "Usage: " << argv[0] << " path/to/systemlm-unigram.trie" << std::endl;
        return 1;
    }

    assert(argv[1] && " should be non-null.");

    akaza::SystemUnigramLM lm;
    lm.load(argv[1]);
    lm.dump();
}
