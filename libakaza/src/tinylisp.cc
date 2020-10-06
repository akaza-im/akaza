#include <memory>
#include "../include/akaza.h"

using namespace akaza::tinylisp;

std::shared_ptr<Node> TinyLisp::eval(std::shared_ptr<Node> x) {
    if (x->type() == NODE_SYMBOL) {
        std::string symbol = dynamic_cast<SymbolNode *>(&*x)->symbol();
        if (symbol == "current-datetime") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_current_datetime));
        } else if (symbol == "strftime") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_strftime));
        } else if (symbol == ".") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_string_concat));
        } else {
            throw std::runtime_error(std::string("Unknown function: ") + symbol);
        }
    } else if (x->type() == NODE_LIST) { // (proc exp*)
        std::vector<std::shared_ptr<Node>> exps;
        for (auto &exp : *dynamic_cast<ListNode *>(&*x)->children()) {
            exps.push_back(eval(exp));
        }

        std::function<std::shared_ptr<Node>(std::vector<std::shared_ptr<Node>> &)>
                glambda = [](std::vector<std::shared_ptr<Node>> &exp) {
            return std::shared_ptr<Node>(new StringNode("HAHAH"));
        };

        auto proc = exps[0];
        exps.erase(exps.begin());
        return dynamic_cast<FunctionNode *>(&*proc)->call(exps);
    } else {
        return x;
    }
}
