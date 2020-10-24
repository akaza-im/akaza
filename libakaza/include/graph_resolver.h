#ifndef LIBAKAZA_GRAPH_RESOLVER_H
#define LIBAKAZA_GRAPH_RESOLVER_H

#include <vector>
#include <memory>
#include <tuple>
#include <optional>

namespace akaza {
    class UserLanguageModel;

    class SystemUnigramLM;

    class SystemBigramLM;

    class BinaryDict;

    class Node;

    class Graph;

    class Slice {
    private:
        const size_t start_;
        const size_t len_;
    public:
        Slice(size_t start, size_t len) : start_(start), len_(len) {
        }

        [[nodiscard]] size_t start() const {
            return start_;
        }

        [[nodiscard]] size_t len() const {
            return len_;
        }

        std::string repr() const;

    };

    /*
     * ビタビアルゴリズムで候補を求める。
     */
    class GraphResolver {
    private:
        std::shared_ptr<UserLanguageModel> user_language_model_;
        std::shared_ptr<SystemUnigramLM> system_unigram_lm_;
        std::shared_ptr<SystemBigramLM> system_bigram_lm_;
        std::vector<std::shared_ptr<BinaryDict>> normal_dicts_;
        std::vector<std::shared_ptr<BinaryDict>> single_term_dicts_;

        std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
        construct_normal_graph(const std::wstring &ws);

        std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
        force_selected_graph(const std::wstring &s, const std::vector<Slice> &force_selected_clauses);

    public:
        GraphResolver(const std::shared_ptr<UserLanguageModel> &user_language_model,
                      const std::shared_ptr<SystemUnigramLM> &system_unigram_lm,
                      const std::shared_ptr<SystemBigramLM> &system_bigram_lm,
                      const std::vector<std::shared_ptr<BinaryDict>> &normal_dicts,
                      const std::vector<std::shared_ptr<BinaryDict>> &single_term_dicts
        );

        Graph graph_construct(const std::wstring &s, std::optional<std::vector<Slice>> force_selected_clause);

        void fill_cost(Graph &graph);

        std::vector<std::vector<std::shared_ptr<akaza::Node>>> find_nbest(akaza::Graph &graph);

        friend class Akaza;
    };
}

#endif //LIBAKAZA_GRAPH_RESOLVER_H
