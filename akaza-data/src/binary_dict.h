#include <cstring>
#include <iostream>
#include <string>
#include <vector>
#include <sstream>

#include "debug.h"
#include "nanoutf8.h"
#include <marisa.h>


namespace akaza {
class BinaryDict {
private:
  marisa::Trie dict_trie;
   std::vector<std::string> split(const std::string &s) {
        std::vector<std::string> elems;
        std::stringstream ss(s);
        std::string item;
        while (getline(ss, item, '/')) {
        if (!item.empty()) {
                elems.push_back(item);
            }
        }
        return elems;
   }

public:
  BinaryDict() {}

  void load(std::string dict_path) {
    dict_trie.load(dict_path.c_str());
    std::cout << dict_path << ": " << dict_trie.num_keys() << std::endl;
  }

  std::vector<std::string> find_kanjis(std::string word) {
    std::string query(word);
    query += "\xff"; // add marker

    marisa::Agent agent;
    agent.set_query(query.c_str(), query.size());

    while (dict_trie.predictive_search(agent)) {
      std::string kanjis = std::string(agent.key().ptr() + query.size(), agent.key().length()-query.size());
      return split(kanjis);
    }
    return std::vector<std::string>();
  }

  // https://github.com/pytries/marisa-trie/blob/57844a6cb96264ebc8a5a3393b5a5723734e0dd1/src/marisa_trie.pyx#L548-L572
  std::vector<std::string> prefixes(std::string src) {
    std::vector<std::string> retval;
    marisa::Agent agent;

    for (size_t i=0; i<src.size();) {
        size_t t = nanoutf8_byte_count_from_first_char(src[i]);
        i += t;
        std::string query(src.substr(0, i) + "\xff");
        // std::cout << "<t=" << t << ":i=" << i<< ":q="<< src.substr(0, i) << ">" << std::endl;

        agent.set_query(query.c_str(), query.size());
        while (dict_trie.predictive_search(agent)) {
          // dump_string(std::string(agent.key().ptr(), agent.key().length()));
          std::string term = std::string(agent.key().ptr(), agent.key().length());
        // std::cout << "<" << term << ">" << std::endl;
          size_t pos = term.find("\xff");
          retval.push_back(term.substr(0, pos));
        }
    }
    return retval;
  }
};
} // namespace akaza
