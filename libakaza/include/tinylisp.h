#ifndef LIBAKAZA_TINYLISP_H_
#define LIBAKAZA_TINYLISP_H_

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
            NodeType type_;

        protected:
            Node(NodeType type) { this->type_ = type; }

        public:
            virtual ~Node() = default;

            NodeType type() { return this->type_; }
            // TODO Implement method like `as_ptr`
        };

        class ListNode : public Node {
        private:
            std::vector<std::shared_ptr<Node>> children_;

        public:
            ListNode(std::vector<std::shared_ptr<Node>> children) : Node(NODE_LIST) {
                this->children_ = children;
            }

            std::vector<std::shared_ptr<Node>> *children() { return &children_; }
        };

        class StringNode : public Node {
        private:
            std::wstring str_;

        public:
            StringNode(const std::wstring &str) : Node(NODE_STRING) { this->str_ = str; }

            std::wstring str() { return str_; }
        };

        class SymbolNode : public Node {
        private:
            std::wstring symbol_;

        public:
            SymbolNode(const std::wstring &symbol) : Node(NODE_SYMBOL) { this->symbol_ = symbol; }

            std::wstring symbol() { return symbol_; }
        };

        using function_node_func =
        std::shared_ptr<Node>(std::vector<std::shared_ptr<Node>> &);

        class FunctionNode : public Node {
        private:
            function_node_func *cb_;

        public:
            FunctionNode(function_node_func *cb) : Node(NODE_FUNCTION) { this->cb_ = cb; }

            std::shared_ptr<Node> call(std::vector<std::shared_ptr<Node>> &exps) {
                return cb_(exps);
            }
        };

        class PointerNode : public Node {
        private:
            void *ptr_;

        public:
            PointerNode(void *ptr) : Node(NODE_POINTER) { this->ptr_ = ptr; }

            void *ptr() { return ptr_; }
        };


        class TinyLisp {
        private:
            static std::vector<std::wstring> split(const std::wstring &s) {
                std::vector<std::wstring> elems;
                std::wstringstream ss(s);
                std::wstring item;
                while (getline(ss, item, L' ')) {
                    if (!item.empty()) {
                        elems.push_back(item);
                    }
                }
                return elems;
            }

            std::wstring replace_string(std::wstring target, std::wstring pattern,
                                        std::wstring format) const {
                std::string::size_type pos(target.find(pattern));

                while (pos != std::string::npos) {
                    target.replace(pos, pattern.length(), format);
                    pos = target.find(pattern, pos + format.length());
                }

                return target;
            }

            std::vector<std::wstring> tokenize(const std::wstring &src) const {
                std::wstring buf = src;
                buf = replace_string(buf, L"(", L" ( ");
                buf = replace_string(buf, L")", L" ) ");
                return split(buf);
            }

            std::shared_ptr<Node> _read_from(std::vector<std::wstring> &tokens,
                                             int depth) const;

            static std::shared_ptr<Node> _atom(const std::wstring &token) ;

        public:
            std::shared_ptr<Node> parse(const std::wstring &src) const {
                auto tokens = tokenize(src);
                return _read_from(tokens, 0);
            }

            std::shared_ptr<Node> eval(std::shared_ptr<Node> x) const;

            std::wstring run(const std::wstring &sexp) const {
                std::shared_ptr<Node> node = this->run_node(sexp);
                return static_cast<StringNode *>(&*node)->str();
            }

            std::shared_ptr<Node> run_node(const std::wstring &sexp) const {
                return this->eval(parse(sexp));
            }
        };

    } // namespace tinylisp
} // namespace akaza

#endif // LIBAKAZA_TINYLISP_H_
