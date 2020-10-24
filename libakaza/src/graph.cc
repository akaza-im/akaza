#include "../include/graph.h"
#include "../include/node.h"
#include "debug_log.h"

#include <iostream>
#include <algorithm>
#include <stdexcept>
#include <cassert>

void akaza::Graph::dump() {
    std::wcout << "# GRAPH --" << std::endl;
    for (const auto &node: nodes_) {
        std::wcout << node->get_start_pos() << "\t" << node->get_key() << "\t\tprev="
                   << (node->get_prev() == nullptr ? L"NULL" : node->get_prev()->get_key())
                   << "\tcost=" << node->get_total_cost() << "\tbigram={";
        for (const auto &[k, v]: node->bigram_cache_) {
            std::wcout << k << "->" << v << ", ";
        }
        std::wcout << "}" << std::endl;
    }
    std::wcout << "# /GRAPH --" << std::endl;
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
            end_pos2nodes_[node->get_start_pos() + node->get_yomi().length()].push_back(node);
        }
    }
    end_pos2nodes_[0].push_back(this->get_bos());

    std::sort(this->nodes_.begin(), this->nodes_.end(),
              [](const std::shared_ptr<Node> &a, const std::shared_ptr<Node> &b) {
                  return a->get_start_pos() < b->get_start_pos();
              });
}

std::vector<std::shared_ptr<akaza::Node>> akaza::Graph::get_prev_items(const std::shared_ptr<Node> &target_node) {
    return end_pos2nodes_[target_node->get_start_pos()];
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

std::shared_ptr<akaza::Node> akaza::Graph::get_eos() {
    for (const auto &node: nodes_) {
        if (node->is_eos()) {
            return node;
        }
    }
    throw std::runtime_error("Missing EOS node in the graph");
}

std::shared_ptr<akaza::Node> akaza::Graph::get_bos() {
    for (const auto &node: nodes_) {
        if (node->is_bos()) {
            return node;
        }
    }
    throw std::runtime_error("Missing BOS node in the graph");
}
