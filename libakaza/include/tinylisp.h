#pragma once

#include <functional>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>
#include <ctime>
#include <cassert>

// 簡易 LISP のインタープリタ for akaza。
// No variables.
// ref. http://norvig.com/lispy.html

namespace akaza {
    namespace tinylisp {

        enum NodeType {
            NODE_LIST, NODE_SYMBOL, NODE_STRING, NODE_FUNCTION, NODE_POINTER
        };

        class Node {
        private:
            NodeType _type;

        protected:
            Node(NodeType type) { this->_type = type; }

        public:
            virtual ~Node() = default;

            NodeType type() { return this->_type; }
            // TODO Implement method like `as_ptr`
        };

        class ListNode : public Node {
        private:
            std::vector<std::shared_ptr<Node>> _children;

        public:
            ListNode(std::vector<std::shared_ptr<Node>> children) : Node(NODE_LIST) {
                this->_children = children;
            }

            std::vector<std::shared_ptr<Node>> *children() { return &_children; }
        };

        class StringNode : public Node {
        private:
            std::string _str;

        public:
            StringNode(std::string str) : Node(NODE_STRING) { this->_str = str; }

            std::string str() { return _str; }
        };

        class SymbolNode : public Node {
        private:
            std::string _symbol;

        public:
            SymbolNode(std::string symbol) : Node(NODE_SYMBOL) { this->_symbol = symbol; }

            std::string symbol() { return _symbol; }
        };

        using function_node_func =
        std::shared_ptr<Node>(std::vector<std::shared_ptr<Node>> &);

        class FunctionNode : public Node {
        private:
            function_node_func *cb;

        public:
            FunctionNode(function_node_func *cb) : Node(NODE_FUNCTION) { this->cb = cb; }

            std::shared_ptr<Node> call(std::vector<std::shared_ptr<Node>> &exps) {
                return cb(exps);
            }
        };

        class PointerNode : public Node {
        private:
            void *_ptr;

        public:
            PointerNode(void *ptr) : Node(NODE_POINTER) { this->_ptr = ptr; }

            void *ptr() { return _ptr; }
        };


        class TinyLisp {
        private:
            static std::vector<std::string> split(const std::string &s) {
                std::vector<std::string> elems;
                std::stringstream ss(s);
                std::string item;
                while (getline(ss, item, ' ')) {
                    if (!item.empty()) {
                        elems.push_back(item);
                    }
                }
                return elems;
            }

            std::string replace_string(std::string target, std::string pattern,
                                       std::string format) const {
                std::string::size_type pos(target.find(pattern));

                while (pos != std::string::npos) {
                    target.replace(pos, pattern.length(), format);
                    pos = target.find(pattern, pos + format.length());
                }

                return target;
            }

            std::vector<std::string> tokenize(std::string src) const {
                src = replace_string(src, "(", " ( ");
                src = replace_string(src, ")", " ) ");
                return split(src);
            }

            std::shared_ptr<Node> _read_from(std::vector<std::string> &tokens,
                                             int depth) const {
                if (tokens.size() == 0) {
                    throw std::runtime_error("Unexpected EOF while reading(LISP)");
                }
                std::string token = tokens[0];
                tokens.erase(tokens.begin());
                if (token == "(") {
                    std::vector<std::shared_ptr<Node>> values;
                    while (tokens[0] != ")") {
                        values.push_back(_read_from(tokens, depth + 1));
                    }
                    tokens.erase(tokens.begin()); // pop off ")"
                    return std::shared_ptr<Node>(new ListNode(values));
                } else if (token == ")") {
                    throw std::runtime_error("Unexpected ')'");
                } else {
                    return _atom(token);
                }
            }

            std::shared_ptr<Node> _atom(std::string token) const {
                if (token.size() > 0 && token[0] == '"') {
                    return std::shared_ptr<Node>(
                            new StringNode(token.substr(1, token.size() - 2)));
                } else {
                    return std::shared_ptr<Node>(new SymbolNode(token));
                }
            }

        public:
            std::shared_ptr<Node> parse(std::string src) const {
                auto tokens = tokenize(src);
                return _read_from(tokens, 0);
            }

            std::shared_ptr<Node> eval(std::shared_ptr<Node> x) const;

            std::string run(std::string sexp) const {
                std::shared_ptr<Node> node = this->run_node(sexp);
                return static_cast<StringNode *>(&*node)->str();
            }

            std::shared_ptr<Node> run_node(std::string sexp) const {
                return this->eval(parse(sexp));
            }
        };

    } // namespace tinylisp
} // namespace akaza