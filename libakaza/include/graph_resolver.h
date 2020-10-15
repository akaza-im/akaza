#ifndef LIBAKAZA_GRAPH_RESOLVER_H
#define LIBAKAZA_GRAPH_RESOLVER_H

#include <utility>
#include <vector>
#include <memory>
#include <valarray>
#include <tuple>

#include "user_language_model.h"
#include "system_lm.h"
#include "binary_dict.h"
#include "graph.h"

namespace akaza {
    class Slice {
    private:
        size_t _start;
        size_t _len;
    public:
        Slice(size_t start, size_t len) {
            _start = start;
            _len = len;
        }

        [[nodiscard]] size_t start() const {
            return _start;
        }

        [[nodiscard]] size_t len() const {
            return _len;
        }

        std::string repr();

    };

    /*
     * ビタビアルゴリズムで候補を求める。
     */
    class GraphResolver {
        /*
 def __init__(self,
                 user_language_model: UserLanguageModel,
                 system_unigram_lm: SystemUnigramLM,
                 system_bigram_lm: SystemBigramLM,
                 normal_dicts: List[BinaryDict],
                 single_term_dicts: List[BinaryDict]):
        self.user_language_model = user_language_model
        self.system_unigram_lm = system_unigram_lm
        self.system_bigram_lm = system_bigram_lm
        self.normal_dicts = normal_dicts
        self.single_term_dicts = single_term_dicts
         */
    private:
        std::shared_ptr<UserLanguageModel> user_language_model_;
        std::shared_ptr<SystemUnigramLM> _system_unigram_lm;
        std::shared_ptr<SystemBigramLM> _system_bigram_lm;
        std::vector<std::shared_ptr<BinaryDict>> _normal_dicts;
        std::vector<std::shared_ptr<BinaryDict>> _single_term_dicts;

        std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
        construct_normal_graph(const std::wstring &ws);

        std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
        force_selected_graph(const std::string &s, const std::vector<Slice> &force_selected_clauses);

    public:
        GraphResolver(const std::shared_ptr<UserLanguageModel> &user_language_model,
                      const std::shared_ptr<SystemUnigramLM> &system_unigram_lm,
                      const std::shared_ptr<SystemBigramLM> &system_bigram_lm,
                      const std::vector<std::shared_ptr<BinaryDict>> &normal_dicts,
                      const std::vector<std::shared_ptr<BinaryDict>> &single_term_dicts
        );

        /*
    def lookup(self, s: str):
        for i in range(0, len(s)):
            yomi = s[i:]
            # print(f"YOMI:::: {yomi}")
            words = set().union(*[normal_dict.prefixes(yomi) for normal_dict in self.normal_dicts])
            if len(words) > 0:
                # print(f"YOMI:::: {yomi} {words}")
                for word in words:
                    kanjis = list(
                        set().union(*[normal_dict.find_kanjis(word) for normal_dict in self.normal_dicts]))
                    if word not in kanjis:
                        kanjis.append(word)

                    kata = jaconv.hira2kata(word)
                    if kata not in kanjis:
                        kanjis.append(kata)

                    if word == yomi:
                        for single_term_dict in self.single_term_dicts:
                            for emoji in single_term_dict.find_kanjis(yomi):
                                if emoji not in kanjis:
                                    kanjis.append(emoji)

                    yield word, kanjis

                if yomi not in words and self.user_language_model.has_unigram_cost_by_yomi(yomi):
                    # システム辞書に入ってないがユーザー言語モデルには入っているという場合は候補にいれる。
                    kanjis = [yomi]

                    kata = jaconv.hira2kata(yomi)
                    if kata not in kanjis:
                        kanjis.append(kata)
                    for single_term_dict in self.single_term_dicts:
                        for emoji in single_term_dict.find_kanjis(yomi):
                            if emoji not in kanjis:
                                kanjis.append(emoji)

                    yield yomi, kanjis
            else:
                # print(f"YOMI~~~~:::: {yomi}")
                kanjis = [yomi[0]]
                hira = jaconv.hira2kata(yomi[0])
                if hira not in kanjis:
                    kanjis.append(hira)
                for single_term_dict in self.single_term_dicts:
                    for emoji in single_term_dict.find_kanjis(yomi):
                        if emoji not in kanjis:
                            kanjis.append(emoji)
                yield yomi[0], kanjis

         */
        /**
         * lookup - 変換候補を列挙する。
         */

        /*
     # n文字目でおわる単語リストを作成する
    def graph_construct(self, s, ht, force_selected_clause: List[slice] = None) -> Graph:
        graph = Graph(size=len(s))

        if force_selected_clause:
        else:

        return graph


         force_selected_clause: ユーザーが自ら選択した文節を表示する。

         */
        Graph graph_construct(const std::string &s, std::optional<std::vector<Slice>> force_selected_clause);

        void fill_cost(Graph &graph);

        std::vector<std::vector<std::shared_ptr<akaza::Node>>> find_nbest(akaza::Graph &graph);
    };
}

#endif //LIBAKAZA_GRAPH_RESOLVER_H
