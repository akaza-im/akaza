#include <cstring>
#include <iostream>
#include <string>
#include <vector>
#include <sstream>

#include "debug.h"
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

  std::vector<std::string> prefixes(std::string src) {
    std::vector<std::string> retval;

    marisa::Agent agent;
    agent.set_query(src.c_str(), src.size());
    std::cout << "A" << src << "B" << std::endl;

    while (dict_trie.predictive_search(agent)) {
      // dump_string(std::string(agent.key().ptr(), agent.key().length()));
      std::string term = std::string(agent.key().ptr(), agent.key().length());
    std::cout << "<" << term << ">" << std::endl;
      size_t pos = term.find("\xff");
      retval.push_back(term.substr(0, pos));
    }
    return retval;
  }
};
} // namespace akaza
