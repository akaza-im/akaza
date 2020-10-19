#ifndef LIBAKAZA_GRAPH_H
#define LIBAKAZA_GRAPH_H

#include <vector>
#include <memory>
#include "node.h"


namespace akaza {
    class Graph {
    private:
        int size_;
        std::vector<std::shared_ptr<Node>> nodes_;
    public:
        Graph() {
        }

        int size() {
            return size_;
        }

        /*
     def get_items(self):
        for i in sorted(self.d.keys()):
            if i == 0:  # skip bos
                continue
            yield self.d[i]
         */
        std::vector<std::shared_ptr<Node>> get_items() {
            return nodes_;
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
        std::shared_ptr<akaza::Node> get_eos();

        /*
    def get_bos(self):
        return self.d[0][0]
         */
        std::shared_ptr<Node> get_bos();

        void dump();

        void build(int i, const std::vector<std::tuple<int, std::vector<std::shared_ptr<akaza::Node>>>> &vector);
    };
}

#endif //LIBAKAZA_GRAPH_H
