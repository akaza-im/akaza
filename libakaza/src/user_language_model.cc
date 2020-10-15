#include "../include/akaza.h"

#include "split.h"

inline int my_atoi(const std::string &s) {
    std::stringstream ss(s);
    int n;
    ss >> n;
    return n;
}

void akaza::UserLanguageModel::read(const std::string &path, bool is_unigram, int &c, int &v,
                                    std::map<std::string, int> &map) {
/*
        word_data = {}
        with open(path) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    key, count = m
                    count = int(count)
                    word_data[key] = count
                    if is_unigram:
                        kanji, kana = key.split('/')
                        self.unigram_kanas.add(kana)
                    V += 1
                    C += count
        return V, C, word_data
 */
    c = 0;
    v = 0;

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    std::ifstream ifs(path, std::ifstream::in);
    std::string line;
    while (std::getline(ifs, line)) {
        bool splitted;
        std::tuple<std::string, std::string> m = split2(line, ' ', splitted);
        if (!splitted) {
            continue;
        }
        auto key = std::get<0>(m);
        int count = my_atoi(std::get<1>(m));
        map[key] = count;
        if (is_unigram) {
            auto kana = std::get<1>(split2(line, '/', splitted));
            if (splitted) {
                unigram_kanas.insert(cnv.from_bytes(kana));
            }
        }
        v += 1;
        c += count;
    }
}

/*
def add_entry(self, nodes: List[Node]):
    # unigram
    for node in nodes:
        key = node.get_key()

        self.logger.info(f"add user_language_model entry: key={key}")

        if key not in self.unigram:
            self.unigram_C += 1
        self.unigram_V += 1
        kanji, kana = key.split('/')
        self.unigram_kanas.add(kana)
        self.unigram[key] = self.unigram.get(key, 0) + 1

    # bigram
    for i in range(1, len(nodes)):
        node1 = nodes[i - 1]
        node2 = nodes[i]
        key = node1.get_key() + "\t" + node2.get_key()
        if key not in self.bigram:
            self.bigram_C += 1
        self.bigram_V += 1
        self.bigram[key] = self.bigram.get(key, 0) + 1

    self.need_save = True
 */
void akaza::UserLanguageModel::add_entry(std::vector<Node> nodes) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv; // TODO remove

    // unigram
    for (auto &node: nodes) {
        auto key = cnv.to_bytes(node.get_key());
        if (unigram.count(key) == 0) {
            unigram_C += 1;
        }
        unigram_V += 1;
        bool splitted;
        auto kana = std::get<1>(split2(key, '/', splitted));
        unigram_kanas.insert(cnv.from_bytes(kana));
        unigram[key] = unigram.count(key) > 0 ? unigram[key] + 1 : 1;
    }

    // bigram
    for (int i = 1; i < nodes.size(); i++) {
        auto &node1 = nodes[i - 1];
        auto &node2 = nodes[i];

        auto key = cnv.to_bytes(node1.get_key() + L"\t" + node2.get_key());
        if (bigram.count(key) == 0) {
            bigram_C += 1;
        }
        bigram_V += 1;
        bigram[key] = unigram.count(key) > 0 ? unigram[key] + 1 : 1;
    }

    need_save = true;
}
