#include "../include/akaza.h"
#include "nanoutf8.h"

// https://github.com/pytries/marisa-trie/blob/57844a6cb96264ebc8a5a3393b5a5723734e0dd1/src/marisa_trie.pyx#L548-L572
std::vector<std::string> akaza::BinaryDict::prefixes(std::string src) {
    std::vector<std::string> retval;
    marisa::Agent agent;

    for (size_t i = 0; i < src.size();) {
        size_t t = nanoutf8_byte_count_from_first_char(src[i]);
        i += t;
        std::string query(src.substr(0, i) + "\xff");
        // std::cout << "<t=" << t << ":i=" << i<< ":q="<< src.substr(0, i) << ">" << std::endl;

        agent.set_query(query.c_str(), query.size());
        while (dict_trie.predictive_search(agent)) {
            // dump_string(std::string(agent.key().ptr(), agent.key().length()));
            std::string term = std::string(agent.key().ptr(), agent.key().length());
            // std::cout << "<" << term << ">" << std::endl;
            size_t pos = term.find('\xff');
            retval.push_back(term.substr(0, pos));
        }
    }
    return retval;
}
