#ifndef LIBAKAZA_GRAPH_H
#define LIBAKAZA_GRAPH_H

#include <vector>
#include <memory>
#include <unordered_map>


namespace akaza {
    class Node;

    class Graph {
    private:
        int size_;
        std::vector<std::shared_ptr<Node>> nodes_;
        // key: node->get_start_pos() + node->get_yomi().length()
        // value: node
        std::unordered_map<int, std::vector<std::shared_ptr<Node>>> end_pos2nodes_;
    public:
        Graph() {
        }

        int size() {
            return size_;
        }

        std::vector<std::shared_ptr<Node>> get_items() {
            return nodes_;
        }

        std::vector<std::shared_ptr<Node>> get_items_by_start_and_length(const std::shared_ptr<Node> &node);

        std::vector<std::shared_ptr<Node>> get_prev_items(const std::shared_ptr<Node> &node);

        std::shared_ptr<Node> get_eos();

        std::shared_ptr<Node> get_bos();

        void dump();

        void build(int i, const std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> &vector);
    };
}

#endif //LIBAKAZA_GRAPH_H
