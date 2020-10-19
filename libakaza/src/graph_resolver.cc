#include "../include/graph_resolver.h"
#include "../include/binary_dict.h"
#include "../include/user_language_model.h"
#include "../include/system_lm.h"
#include "../include/node.h"
#include "../include/graph.h"
#include "debug_log.h"
#include "kana.h"

#include <memory>
#include <locale>
#include <cassert>
#include <algorithm>
#include <set>
#include <sstream>
#include <iostream>

akaza::GraphResolver::GraphResolver(const std::shared_ptr<UserLanguageModel> &user_language_model,
                                    const std::shared_ptr<SystemUnigramLM> &system_unigram_lm,
                                    const std::shared_ptr<SystemBigramLM> &system_bigram_lm,
                                    const std::vector<std::shared_ptr<BinaryDict>> &normal_dicts,
                                    const std::vector<std::shared_ptr<BinaryDict>> &single_term_dicts) {
    user_language_model_ = user_language_model;
    system_unigram_lm_ = system_unigram_lm;
    system_bigram_lm_ = system_bigram_lm;
    normal_dicts_ = normal_dicts;
    single_term_dicts_ = single_term_dicts;

    D(std::cout << "GraphResolver: "
                << " ULM.uni=" << user_language_model_->size_unigram()
                << " ULM.bi=" << user_language_model->size_bigram()
                << " SystemUnigramLM.size=" << system_unigram_lm->size()
                << " SystemBigramLM.size=" << system_bigram_lm->size());
    for (const auto &d: normal_dicts) {
        D(std::cout << " ND=" << d->size());
    }
    for (const auto &d: single_term_dicts) {
        D(std::cout << " STD=" << d->size());
    }
    D(std::cout << std::endl);
}

static inline void insert_basic_candidates(std::set<std::tuple<std::wstring, std::wstring>> &kanjiset,
                                           const std::wstring &yomi) {
    kanjiset.insert(std::make_tuple(yomi, yomi));
    kanjiset.insert(std::make_tuple(yomi, akaza::hira2kata(yomi)));
    // TODO: 半角 alphabet 候補もいれたいかも？
    // TODO: 全角 alphabet 候補もいれたいかも？
}

std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
akaza::GraphResolver::construct_normal_graph(const std::wstring &ws) {
    std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> src;

    for (int i = 0; i < ws.size(); i++) {
        std::set<std::tuple<std::wstring, std::wstring>> kanjiset;
        for (int j = 1; j <= ws.size() - i; j++) {
            std::wstring yomi = ws.substr(i, j);

            bool exist_kanjis = false;

            // 通常の辞書から検索してみる
            for (const auto &normal_dict: normal_dicts_) {
                auto kanjis = normal_dict->find_kanjis(yomi);
                for (auto &kanji: kanjis) {
                    kanjiset.insert(std::make_tuple(yomi, kanji));
                    exist_kanjis = true;
                }
            }

            if (exist_kanjis || user_language_model_->has_unigram_cost_by_yomi(yomi)) {
                insert_basic_candidates(kanjiset, yomi);
            }

            // 選択範囲が、文全体であった場合は単文節辞書を参照する。
            if (i == 0 && ws.size() == j) {
                for (const auto &single_term_dict: single_term_dicts_) {
                    std::vector<std::wstring> kanjis = single_term_dict->find_kanjis(yomi);
                    for (auto &kanji: kanjis) {
                        kanjiset.insert(std::make_tuple(yomi, kanji));
                    }
                }

                // 候補がない場合は、Basic 候補をいれていく。
                if (kanjiset.empty()) {
                    insert_basic_candidates(kanjiset, yomi);
                }
            }
        }

        std::vector<std::shared_ptr<akaza::Node>> nodes;
        nodes.reserve(kanjiset.size());
        for (const auto &[yomi, kanji]: kanjiset) {
            nodes.push_back(std::make_shared<akaza::Node>(i, yomi, kanji));
        }
        src.emplace_back(i, nodes);
    }
    return src;
}

std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
akaza::GraphResolver::force_selected_graph(const std::wstring &ws, const std::vector<akaza::Slice> &slices) {
    std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> retval;
    for (const auto &slice : slices) {
        std::set<std::tuple<std::wstring, std::wstring>> kanjiset;

        std::wstring wyomi = ws.substr(slice.start(), slice.len());

        // 通常の辞書から検索してみる
        for (const auto &normal_dict: normal_dicts_) {
            auto kanjis = normal_dict->find_kanjis(wyomi);
            for (auto &kanji: kanjis) {
                kanjiset.insert(std::make_tuple(wyomi, kanji));
            }
        }
        if (wyomi.size() == slice.len()) { // 全部はいってる。
            for (const auto &single_term_dict: single_term_dicts_) {
                auto kanjis = single_term_dict->find_kanjis(wyomi);
                for (auto &kanji: kanjis) {
                    kanjiset.insert(std::make_tuple(wyomi, kanji));
                }
            }

        }

        insert_basic_candidates(kanjiset, wyomi);

        std::vector<std::shared_ptr<akaza::Node>> nodes;
        nodes.reserve(kanjiset.size());
        for (const auto &[yomi, kanji]: kanjiset) {
            nodes.push_back(std::make_shared<akaza::Node>(slice.start(), yomi, kanji));
        }
        retval.emplace_back(slice.start(), nodes);
    }
    return retval;
}

void akaza::GraphResolver::fill_cost(akaza::Graph &graph) {
    for (const auto &node: graph.get_items()) {
        if (node->is_bos()) {
            continue;
        }
        D(std::wcout << "fill_cost: " << node->get_key() << std::endl);
        float node_cost = node->calc_node_cost(*user_language_model_, *system_unigram_lm_);
        float cost = INT32_MIN;
        std::vector<std::shared_ptr<akaza::Node>> prev_nodes = graph.get_prev_items(node);

        if (!prev_nodes.empty()) {
            std::shared_ptr<Node> shortest_prev;
            for (const auto &prev_node: prev_nodes) {
//                D(std::cout << "set prev: " << node->get_key() << " " << prev_node->get_key()
//                            << " " << __FILE__ << ":" << __LINE__ << std::endl);
                float bigram_cost = prev_node->get_bigram_cost(
                        *node,
                        *user_language_model_,
                        *system_bigram_lm_);
                float prev_cost = prev_node->get_cost();
                float tmp_cost = prev_cost + bigram_cost + node_cost;
                if (cost < tmp_cost) { // コストが最大になる経路をえらんでいる
                    cost = tmp_cost;
                    shortest_prev = prev_node;
                }
            }
            assert(shortest_prev);
            D(std::wcout << "[fill_cost] set prev: " << node->get_key() << " " << shortest_prev->get_key()
                         << " " << __FILE__ << ":" << __LINE__ << std::endl);
            node->set_prev(shortest_prev);
            node->set_cost(cost);
        } else {
            D(std::wcout << "\tno prev: " << node->get_key() << std::endl);
            node->set_cost(cost);
        }
    }
}

std::vector<std::vector<std::shared_ptr<akaza::Node>>> akaza::GraphResolver::find_nbest(akaza::Graph &graph) {
    std::shared_ptr<akaza::Node> node = graph.get_eos()->get_prev();
    assert(node != nullptr);

    std::vector<std::vector<std::shared_ptr<akaza::Node>>> result;
    std::shared_ptr<akaza::Node> last_node = graph.get_eos();
    while (!node->is_bos()) {
        if (node == node->get_prev()) {
            throw std::runtime_error("invalid state");
        }

        std::vector<std::shared_ptr<akaza::Node>> nodes = graph.get_items_by_start_and_length(node);
        const auto &userLanguageModel = this->user_language_model_;
        const auto &systemBigramLm = this->system_bigram_lm_;
        std::sort(nodes.begin(), nodes.end(), [last_node, userLanguageModel, systemBigramLm](auto &a, auto &b) {
            return a->get_cost() + a->get_bigram_cost_from_cache(*last_node, *systemBigramLm)
                   > b->get_cost() + b->get_bigram_cost_from_cache(*last_node, *systemBigramLm);
        });

        result.push_back(nodes);

        last_node = node;
        node = node->get_prev();
    }
    std::reverse(result.begin(), result.end());

    return result;
}

akaza::Graph
akaza::GraphResolver::graph_construct(const std::wstring &ws, std::optional<std::vector<Slice>> force_selected_clause) {

    Graph graph = Graph();
    auto nodemap = force_selected_clause.has_value()
                   ? force_selected_graph(ws, force_selected_clause.value())
                   : construct_normal_graph(ws);
    graph.build(ws.size(), nodemap);
    return graph;
}

std::string akaza::Slice::repr() const {
    std::stringstream ss;
    ss << "<akaza::Slice start=" << start_ << " len=" << len_ << ">";
    return ss.str();
}
