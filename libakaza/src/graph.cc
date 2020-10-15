#include <locale>
#include <codecvt>
#include "../include/akaza.h"
#include "../include/graph.h"
#include "debug_log.h"

void akaza::Graph::dump() {
    std::cout << "# GRAPH --" << std::endl;
    for (const auto &node: nodes_) {
        std::wcout << node->get_start_pos() << "\t" << node->get_key() << "\t\t"
                   << (node->get_prev() == nullptr ? L"NULL" : node->get_prev()->get_key())
                   << "\t" << node->get_cost()
                   << std::endl;
    }
    std::cout << "# /GRAPH --" << std::endl;
}

// nodmap は、start_pos にたいして処理されていく。
void
akaza::Graph::build(int size,
                    const std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> &nodemap) {
    this->size_ = size;

    this->nodes_.push_back(akaza::create_bos_node());
    this->nodes_.push_back(akaza::create_eos_node(size));
    for (const auto&[n, nodes]: nodemap) {
        for (const auto &node: nodes) {
            // D(std::cout << "Graph::build-- " << node->get_key() << std::endl);
            this->nodes_.push_back(node);
        }
    }

    std::sort(this->nodes_.begin(), this->nodes_.end(),
              [](const std::shared_ptr<Node> &a, const std::shared_ptr<Node> &b) {
                  return a->get_start_pos() < b->get_start_pos();
              });
}

std::vector<std::shared_ptr<akaza::Node>> akaza::Graph::get_prev_items(const std::shared_ptr<Node> &target_node) {
    if (target_node->get_start_pos() == 0) {
        return {this->get_bos()};
    }

    std::vector<std::shared_ptr<akaza::Node>> nodes;
    for (const auto &node: this->nodes_) {
        if (node->is_bos()) {
            continue;
        }
        if (target_node->is_eos()) {
            if (node->get_key() == L"です/です") {
                D(std::cout << "DDDDD: " << node->get_start_pos() << "\t"
                            << node->get_yomi().length() <<
                            "\t" <<
                            target_node->get_start_pos() << std::endl);
            }
            if (node->get_start_pos() + node->get_yomi().length() ==
                target_node->get_start_pos()) {
                nodes.push_back(node);
            }
        } else {
            if (node->get_start_pos() + node->get_yomi().length() ==
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
    for (const auto &node: this->nodes_) {
        if (node->get_start_pos() == target_node->get_start_pos() &&
            node->get_yomi().length() == target_node->get_yomi().length()) {
            nodes.push_back(node);
        }
    }
    return nodes;
}
