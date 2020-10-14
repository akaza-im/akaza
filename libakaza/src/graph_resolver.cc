#include <memory>
#include <codecvt>
#include <locale>

#include "../include/akaza.h"
#include "debug_log.h"
#include "kana.h"

static std::string tojkata(const std::string &src) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    std::wstring wstr = cnv.from_bytes(src);
    std::transform(wstr.begin(), wstr.end(), wstr.begin(), [](wchar_t c) {
        return std::towctrans(c, std::wctrans("tojkata"));
    });
    std::string sstr = cnv.to_bytes(wstr);
    D(std::cout << "TOJKATA: " << src << " -> " << sstr
                << " " << __FILE__ << ":" << __LINE__ << std::endl);
    return sstr;
}

static inline void insert_basic_candidates(std::set<std::tuple<std::string, std::string>> &kanjiset,
                                           const std::string &yomi) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    kanjiset.insert(std::make_tuple(yomi, yomi));
    kanjiset.insert(std::make_tuple(yomi, cnv.to_bytes(akaza::hira2kata(cnv.from_bytes(yomi)))));
    // TODO: 半角 alphabet 候補もいれたいかも？
    // TODO: 全角 alphabet 候補もいれたいかも？
}

/**
            for i in range(0, len(s)):
                # print(f"LOOP {i}")
                # for i in range(0, len(s)):
                for j in range(i + 1, len(s) + 1):
                    # substr は「読み」であろう。
                    # word は「漢字」であろう。
                    yomi = s[i:j]
                    if yomi in ht:
                        # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                        for kanji in ht[yomi]:
                            node = Node(i, kanji, yomi)
                            graph.append(index=j, node=node)
                    else:
                        if self.user_language_model.has_unigram_cost_by_yomi(yomi):
                            for word in [yomi, jaconv.hira2kata(yomi), jaconv.kana2alphabet(yomi),
                                         jaconv.h2z(jaconv.kana2alphabet(yomi), ascii=True)]:
                                node = Node(start_pos=i, word=word, yomi=yomi)
                                graph.append(index=j, node=node)
 */
std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
akaza::GraphResolver::construct_normal_graph(const std::string &s) {
    std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> src;

    std::wstring_convert<std::codecvt_utf8_utf16<char32_t>, char32_t> utf32conv;
    std::u32string s32 = utf32conv.from_bytes(s);

    for (int i = 0; i < s32.size(); i++) {
        std::set<std::tuple<std::string, std::string>> kanjiset;
        for (int j = 1; j <= s32.size() - i; j++) {
            std::u32string yomi32 = s32.substr(i, j);
            std::string yomi = utf32conv.to_bytes(yomi32);

            bool exist_kanjis = false;

            // 通常の辞書から検索してみる
            for (const auto &normal_dict: _normal_dicts) {
                auto kanjis = normal_dict->find_kanjis(yomi);
                for (auto &kanji: kanjis) {
                    kanjiset.insert(std::make_tuple(yomi, kanji));
                    exist_kanjis = true;
                }
            }

            if (exist_kanjis || _user_language_model->has_unigram_cost_by_yomi(yomi)) {
                insert_basic_candidates(kanjiset, yomi);
            }

            // 選択範囲が、文全体であった場合は単文節辞書を参照する。
            if (i == 0 && s32.size() == j) {
                for (const auto &single_term_dict: _single_term_dicts) {
                    std::vector<std::string> kanjis = single_term_dict->find_kanjis(yomi);
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
        for (auto &[yomi, kanji]: kanjiset) {
            nodes.push_back(std::make_shared<akaza::Node>(i, yomi, kanji));
        }
        src.emplace_back(i, nodes);
    }
    return src;
}

/*
 for force_slice in force_selected_clause:
    # 強制的に範囲を指定されている場合。
    # substr は「読み」であろう。
    # word は「漢字」であろう。
    yomi = s[force_slice]
    i = force_slice.start
    j = force_slice.stop
    # print(f"XXXX={s} {force_slice} {yomi}")
    if yomi in ht:
        # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
        for kanji in ht[yomi]:
            node = Node(i, kanji, yomi)
            graph.append(index=j, node=node)
    else:
        # print(f"NO YOMI: {yomi}")
        if len(yomi) == 0:
            raise AssertionError(f"len(yomi) should not be 0. {s}, {force_slice}")
        for word in [yomi, jaconv.hira2kata(yomi), jaconv.kana2alphabet(yomi),
                     jaconv.h2z(jaconv.kana2alphabet(yomi), ascii=True)]:
            node = Node(start_pos=i, word=word, yomi=yomi)
            graph.append(index=j, node=node)
 */
std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>>
akaza::GraphResolver::force_selected_graph(const std::string &s, const std::vector<akaza::Slice> &slices) {
    std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> retval;
    std::wstring_convert<std::codecvt_utf8_utf16<char32_t>, char32_t> utf32conv;
    std::u32string s32 = utf32conv.from_bytes(s);
    for (const auto &slice : slices) {
        std::set<std::tuple<std::string, std::string>> kanjiset;

        std::u32string yomi32 = s32.substr(slice.start(), slice.len());
        std::string yomi = utf32conv.to_bytes(yomi32);

        // 通常の辞書から検索してみる
        for (const auto &normal_dict: _normal_dicts) {
            auto kanjis = normal_dict->find_kanjis(yomi);
            for (auto &kanji: kanjis) {
                kanjiset.insert(std::make_tuple(yomi, kanji));
            }
        }
        if (yomi32.size() == slice.len()) { // 全部はいってる。
            for (const auto &single_term_dict: _single_term_dicts) {
                auto kanjis = single_term_dict->find_kanjis(yomi);
                for (auto &kanji: kanjis) {
                    kanjiset.insert(std::make_tuple(yomi, kanji));
                }
            }

        }

        insert_basic_candidates(kanjiset, yomi);

        std::vector<std::shared_ptr<akaza::Node>> nodes;
        nodes.reserve(kanjiset.size());
        for (auto &[yomi, kanji]: kanjiset) {
            nodes.push_back(std::make_shared<akaza::Node>(slice.start(), yomi, kanji));
        }
        retval.emplace_back(slice.start(), nodes);
    }
    return retval;
}

/*
     def fill_cost(self, graph: Graph):
        """
        Graph の各ノードについて最短のノードをえる。
        """
        # BOS にスコアを設定。
        graph.get_bos().set_cost(0)

        for nodes in graph.get_items():
            # print(f"fFFFF {nodes}")
            for node in nodes:
                # print(f"  PPPP {node}")
                node_cost = node.calc_node_cost(self.user_language_model,
                                                self.system_unigram_lm)
                # print(f"  NC {node.word} {node_cost}")
                cost = -sys.maxsize
                shortest_prev = None
                prev_nodes = graph.get_item(node.get_start_pos())
                if prev_nodes[0].is_bos():
                    node.set_prev(prev_nodes[0])
                    node.set_cost(node_cost)
                else:
                    for prev_node in prev_nodes:
                        bigram_cost = prev_node.get_bigram_cost(node, self.user_language_model,
                                                                self.system_bigram_lm)
                        tmp_cost = prev_node.cost + bigram_cost + node_cost
                        if cost < tmp_cost:  # 。
                            cost = tmp_cost
                            shortest_prev = prev_node
                    # print(f"    SSSHORTEST: {shortest_prev} in {prev_nodes}")
                    node.prev = shortest_prev
                    node.cost = cost
 */
void akaza::GraphResolver::fill_cost(akaza::Graph &graph) {
    for (const auto &node: graph.get_items()) {
        if (node->is_bos()) {
            continue;
        }
        D(std::cout << "fill_cost: " << node->get_key() << std::endl);
        auto node_cost = node->calc_node_cost(*_user_language_model, *_system_unigram_lm);
        auto cost = INT32_MIN;
        auto prev_nodes = graph.get_prev_items(node);

        if (!prev_nodes.empty()) {
            std::shared_ptr<Node> shortest_prev;
            for (const auto &prev_node: prev_nodes) {
//                D(std::cout << "set prev: " << node->get_key() << " " << prev_node->get_key()
//                            << " " << __FILE__ << ":" << __LINE__ << std::endl);
                auto bigram_cost = prev_node->get_bigram_cost(
                        *node,
                        *_user_language_model,
                        *_system_bigram_lm);
                auto tmp_cost = prev_node->get_cost() + bigram_cost + node_cost;
                if (cost < tmp_cost) { // コストが最大になる経路をえらんでいる
                    cost = tmp_cost;
                    shortest_prev = prev_node;
                }
            }
            D(std::cout << "[fill_cost] set prev: " << node->get_key() << " " << shortest_prev->get_key()
                        << " " << __FILE__ << ":" << __LINE__ << std::endl);
            node->set_prev(shortest_prev);
            node->set_cost(cost);
        } else {
            D(std::cout << "\tno prev: " << node->get_key() << std::endl);
            node->set_cost(cost);
        }
    }
}

/*
    def find_nbest(self, graph: Graph):
        # find EOS.
        node = graph.get_eos()

        result = []
        last_node = None
        while not node.is_bos():
            if node == node.prev:
                print(graph)
                raise AssertionError(f"node==node.prev: {node}")

            if not node.is_eos():
                # 他の候補を追加する。
                nodes = sorted(
                    [n for n in graph.get_item(node.start_pos + len(node.yomi)) if node.yomi == n.yomi],
                    key=lambda x: x.cost + x.get_bigram_cost(last_node,
                                                             self.user_language_model,
                                                             self.system_bigram_lm), reverse=True)
                result.append(nodes)

            last_node = node
            node = node.prev
        return list(reversed(result))
 */
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
        auto userLanguageModel = this->_user_language_model;
        auto systemBigramLm = this->_system_bigram_lm;
        std::sort(nodes.begin(), nodes.end(), [last_node, userLanguageModel, systemBigramLm](auto &a, auto &b) {
            return a->get_cost() + a->get_bigram_cost(*last_node, *userLanguageModel,
                                                      *systemBigramLm)
                   > b->get_cost() + b->get_bigram_cost(*last_node, *userLanguageModel,
                                                        *systemBigramLm);
        });

        result.push_back(nodes);

        last_node = node;
        node = node->get_prev();
    }
    std::reverse(result.begin(), result.end());

    return result;
}

akaza::Graph
akaza::GraphResolver::graph_construct(const std::string &s, std::optional<std::vector<Slice>> force_selected_clause) {
    std::wstring_convert<std::codecvt_utf8_utf16<char32_t>, char32_t> utf32conv;

    Graph graph = Graph();
    auto nodemap = force_selected_clause.has_value()
                   ? force_selected_graph(s, force_selected_clause.value())
                   : construct_normal_graph(s);
    graph.build(utf32conv.from_bytes(s).size(), nodemap);
    return graph;
}
