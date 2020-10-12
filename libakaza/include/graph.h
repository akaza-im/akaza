#ifndef LIBAKAZA_GRAPH_H
#define LIBAKAZA_GRAPH_H

#include <cstddef>
#include <utility>
#include <vector>
#include <memory>
#include "node.h"


namespace akaza {
    class Graph {
    private:
        int _size;
        std::vector<std::shared_ptr<Node>> _nodes;
    public:
        Graph() {
        }

        int size() {
            return _size;
        }

        /*
     def append(self, index: int, node: Node) -> None:
        if index not in self.d:
            self.d[index] = []
        # print(f"graph[{j}]={graph[j]} graph={graph}")
        self.d[index].append(node)

         */
        // hmmmm.....?
        // TODO remove me
        // void append(std::vector<std::shared_ptr<Node>> nodes);

        /*
     def get_items(self):
        for i in sorted(self.d.keys()):
            if i == 0:  # skip bos
                continue
            yield self.d[i]
         */
        std::vector<std::shared_ptr<Node>> get_items() {
            return _nodes;
        }

        std::vector<std::shared_ptr<Node>> get_items_by_start_and_length(const std::shared_ptr<Node> &node);

        /*
     def get_item(self, i: int) -> List[Node]:
        return self.d[i]
         */
        std::vector<std::shared_ptr<Node>> get_prev_items(const std::shared_ptr<Node> &node);

        /*
    def get_eos(self):
        return self.d[max(self.d.keys())][0]
         */
        std::shared_ptr<akaza::Node> get_eos() {
            for (const auto &node: _nodes) {
                if (node->is_eos()) {
                    return node;
                }
            }
            throw std::runtime_error("Missing EOS node in the graph");
        }

        /*
    def get_bos(self):
        return self.d[0][0]
         */
        std::shared_ptr<Node> get_bos() {
            for (const auto &node: _nodes) {
                if (node->is_bos()) {
                    return node;
                }
            }
            throw std::runtime_error("Missing BOS node in the graph");
        }

        void dump();

        void build(int i, const std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> &vector);
    };
}

#endif //LIBAKAZA_GRAPH_H
