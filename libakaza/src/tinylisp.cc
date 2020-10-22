#include <memory>
#include <codecvt>
#include <locale>

#include "../include/tinylisp.h"

using namespace akaza::tinylisp;

static std::shared_ptr<Node> builtin_string_concat(std::vector<std::shared_ptr<Node>> &expr) {
    auto a = expr[0];
    auto b = expr[1];
    std::wstring a_str = dynamic_cast<StringNode *>(&*a)->str();
    std::wstring b_str = dynamic_cast<StringNode *>(&*b)->str();
    return std::shared_ptr<Node>(new StringNode(a_str + b_str));
}

static std::shared_ptr<Node> builtin_current_datetime(std::vector<std::shared_ptr<Node>> &expr) {
    time_t rawtime;
    time(&rawtime);
    struct tm *timeinfo = localtime(&rawtime);
    return std::shared_ptr<Node>(new PointerNode(timeinfo));
}

static std::shared_ptr<Node> builtin_strftime(std::vector<std::shared_ptr<Node>> &expr) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    auto dt = expr[0];
    auto fmt = expr[1];
    std::wstring fmt_str = dynamic_cast<StringNode *>(&*fmt)->str();
    size_t len = fmt_str.size() * 4;
    char *buffer = new char[len];
    size_t got_len = std::strftime(buffer, len, cnv.to_bytes(fmt_str).c_str(),
                                   static_cast<const struct tm *>(dynamic_cast<PointerNode *>(&*dt)->ptr()));
    auto retval = std::shared_ptr<Node>(new StringNode(cnv.from_bytes(std::string(buffer, got_len))));
    delete[] buffer;
    return retval;
}


std::shared_ptr<Node> TinyLisp::eval(std::shared_ptr<Node> x) const {
    if (x->type() == NODE_SYMBOL) {
        std::wstring symbol = dynamic_cast<SymbolNode *>(&*x)->symbol();
        if (symbol == L"current-datetime") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_current_datetime));
        } else if (symbol == L"strftime") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_strftime));
        } else if (symbol == L".") {
            return std::shared_ptr<Node>(new FunctionNode(builtin_string_concat));
        } else {
            std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
            throw std::runtime_error(cnv.to_bytes(std::wstring(L"Unknown function: ") + symbol));
        }
    } else if (x->type() == NODE_LIST) { // (proc exp*)
        std::vector<std::shared_ptr<Node>> exps;
        for (auto &exp : *dynamic_cast<ListNode *>(&*x)->children()) {
            exps.push_back(eval(exp));
        }

        auto proc = exps[0];
        exps.erase(exps.begin());
        return dynamic_cast<FunctionNode *>(&*proc)->call(exps);
    } else {
        return x;
    }
}

std::shared_ptr<Node>
TinyLisp::_read_from(std::vector<std::wstring> &tokens, int depth, const std::wstring &src) const {
    if (tokens.empty()) {
        std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
        throw std::runtime_error(std::string("Unexpected EOF while reading(LISP): ") + cnv.to_bytes(src));
    }
    std::wstring token = tokens[0];
    tokens.erase(tokens.begin());
    if (token == L"(") {
        std::vector<std::shared_ptr<Node>> values;
        while (tokens[0] != L")") {
            values.push_back(_read_from(tokens, depth + 1, src));
        }
        tokens.erase(tokens.begin()); // pop off ")"
        return std::shared_ptr<Node>(new ListNode(values));
    } else if (token == L")") {
        throw std::runtime_error("Unexpected ')'");
    } else {
        return _atom(token);
    }
}

std::shared_ptr<Node> TinyLisp::_atom(const std::wstring &token) {
    if (!token.empty() && token[0] == '"') {
        return std::shared_ptr<Node>(
                new StringNode(token.substr(1, token.size() - 2)));
    } else {
        return std::shared_ptr<Node>(new SymbolNode(token));
    }
}
