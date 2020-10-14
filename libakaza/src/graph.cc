#include <locale>
#include <codecvt>
#include "../include/akaza.h"
#include "../include/graph.h"
#include "debug_log.h"

void akaza::Graph::dump() {
    std::cout << "# GRAPH --" << std::endl;
    for (const auto &node: _nodes) {
        std::cout << node->get_start_pos() << "\t" << node->get_key() << "\t\t"
                  << (node->get_prev() == nullptr ? "NULL" : node->get_prev()->get_key())
                  << "\t" << node->get_cost()
                  << std::endl;
    }
    std::cout << "# /GRAPH --" << std::endl;
}

// nodmap は、start_pos にたいして処理されていく。
void
akaza::Graph::build(int size,
                    const std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> &nodemap) {
    this->_size = size;

    this->_nodes.push_back(akaza::create_bos_node());
    this->_nodes.push_back(akaza::create_eos_node(size));
    for (const auto&[n, nodes]: nodemap) {
        for (const auto &node: nodes) {
            // D(std::cout << "Graph::build-- " << node->get_key() << std::endl);
            this->_nodes.push_back(node);
        }
    }

    std::sort(this->_nodes.begin(), this->_nodes.end(),
              [](const std::shared_ptr<Node> &a, const std::shared_ptr<Node> &b) {
                  return a->get_start_pos() < b->get_start_pos();
              });
}

std::vector<std::shared_ptr<akaza::Node>> akaza::Graph::get_prev_items(const std::shared_ptr<Node> &target_node) {
    if (target_node->get_start_pos() == 0) {
        return {this->get_bos()};
    }

    std::vector<std::shared_ptr<akaza::Node>> nodes;
    std::wstring_convert<std::codecvt_utf8_utf16<char32_t>, char32_t> utf32conv;
    for (const auto &node: this->_nodes) {
        if (node->is_bos()) {
            continue;
        }
        if (target_node->is_eos()) {
            if (node->get_key() == "です/です") {
                D(std::cout << "DDDDD: " << node->get_start_pos() << "\t"
                            << utf32conv.from_bytes(node->get_yomi()).length() <<
                            "\t" <<
                            target_node->get_start_pos() << std::endl);
            }
            if (node->get_start_pos() + utf32conv.from_bytes(node->get_yomi()).length() ==
                target_node->get_start_pos()) {
                nodes.push_back(node);
            }
        } else {
            if (node->get_start_pos() + utf32conv.from_bytes(node->get_yomi()).length() ==
                target_node->get_start_pos()) {
                assert(!node->is_bos());
                nodes.push_back(node);
            }
        }
    }
    return nodes;
}

/**
 * 引数の node と同じ区間にある候補ノードを抽出する
 */
std::vector<std::shared_ptr<akaza::Node>>
akaza::Graph::get_items_by_start_and_length(const std::shared_ptr<akaza::Node> &target_node) {
    std::vector<std::shared_ptr<akaza::Node>> nodes;
    std::wstring_convert<std::codecvt_utf8_utf16<char32_t>, char32_t> utf32conv;
    for (const auto &node: this->_nodes) {
        if (node->get_start_pos() == target_node->get_start_pos() &&
            node->get_yomi().length() == target_node->get_yomi().length()) {
            nodes.push_back(node);
        }
    }
    return nodes;
}
