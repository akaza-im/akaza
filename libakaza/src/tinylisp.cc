#include <memory>
#include "../include/akaza.h"

using namespace akaza::tinylisp;

static std::shared_ptr<Node> builtin_string_concat(std::vector<std::shared_ptr<Node>> &expr) {
    auto a = expr[0];
    auto b = expr[1];
    std::string a_str = dynamic_cast<StringNode *>(&*a)->str();
    std::string b_str = dynamic_cast<StringNode *>(&*b)->str();
    return std::shared_ptr<Node>(new StringNode(a_str + b_str));
}

static std::shared_ptr<Node> builtin_current_datetime(std::vector<std::shared_ptr<Node>> &expr) {
    time_t rawtime;
    time(&rawtime);
    struct tm *timeinfo = localtime(&rawtime);
    return std::shared_ptr<Node>(new PointerNode(timeinfo));
}

static std::shared_ptr<Node> builtin_strftime(std::vector<std::shared_ptr<Node>> &expr) {
    auto dt = expr[0];
    auto fmt = expr[1];
    std::string fmt_str = dynamic_cast<StringNode *>(&*fmt)->str();
    size_t len = fmt_str.size() * 4;
    char *buffer = new char[len];
    size_t got_len = std::strftime(buffer, len, fmt_str.c_str(),
                                   static_cast<const struct tm *>(dynamic_cast<PointerNode *>(&*dt)->ptr()));
    auto retval = std::shared_ptr<Node>(new StringNode(std::string(buffer, got_len)));
    delete[] buffer;
    return retval;
}


std::shared_ptr<Node> TinyLisp::eval(std::shared_ptr<Node> x) const {
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
