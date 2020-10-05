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

namespace akaza {
namespace tinylisp {

enum NodeType { NODE_LIST, NODE_SYMBOL, NODE_STRING, NODE_FUNCTION, NODE_POINTER };

class Node {
private:
  NodeType _type;

protected:
  Node(NodeType type) { this->_type = type; }

public:
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
  function_node_func* cb;

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
  PointerNode(void* ptr) : Node(NODE_POINTER) { this->_ptr = ptr; }
  void * ptr() { return _ptr; }
};

// TODO move to libakaza.so

static std::shared_ptr<Node> builtin_current_datetime(std::vector<std::shared_ptr<Node>> &expr) {
  time_t rawtime;
  time(&rawtime);
  struct tm* timeinfo = localtime(&rawtime);
  return std::shared_ptr<Node>(new PointerNode(timeinfo));
}

static std::shared_ptr<Node> builtin_strftime(std::vector<std::shared_ptr<Node>> &expr) {
  auto dt = expr[0];
  auto fmt = expr[1];
  std::string fmt_str = static_cast<StringNode*>(&*fmt)->str();
  size_t len = fmt_str.size() * 4;
  char* buffer = new char[len];
  size_t got_len = std::strftime(buffer, len,fmt_str.c_str(), static_cast<const struct tm*>(static_cast<PointerNode*>(&*dt)->ptr()));
  auto retval = std::shared_ptr<Node>(new StringNode(std::string(buffer, got_len)));
  delete [] buffer;
  return retval;
}

class TinyLisp {
private:
  std::vector<std::string> split(const std::string &s) {
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
                             std::string format) {
    std::string::size_type pos(target.find(pattern));

    while (pos != std::string::npos) {
      target.replace(pos, pattern.length(), format);
      pos = target.find(pattern, pos + format.length());
    }

    return target;
  }

  std::vector<std::string> tokenize(std::string src) {
    src = replace_string(src, "(", " ( ");
    src = replace_string(src, ")", " ) ");
    return split(src);
  }

  std::shared_ptr<Node> _read_from(std::vector<std::string> &tokens,
                                   int depth) {
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

  std::shared_ptr<Node> _atom(std::string token) {
    if (token.size() > 0 && token[0] == '"') {
      return std::shared_ptr<Node>(
          new StringNode(token.substr(1, token.size() - 2)));
    } else {
      return std::shared_ptr<Node>(new SymbolNode(token));
    }
  }

public:
  std::shared_ptr<Node> parse(std::string src) {
    auto tokens = tokenize(src);
    return _read_from(tokens, 0);
  }

  std::shared_ptr<Node> eval(std::shared_ptr<Node> x) {
    if (x->type() == NODE_SYMBOL) {
      // RETURN SYMBOL VALUE FROM ENV
      // TODO TODO
      std::string symbol = static_cast<SymbolNode*>(&*x)->symbol();
      if (symbol == "current-datetime") {
        return std::shared_ptr<Node>(new FunctionNode(builtin_current_datetime));
      } else if (symbol == "strftime") {
        return std::shared_ptr<Node>(new FunctionNode(builtin_strftime));
      } else {
        throw std::runtime_error(std::string("Unknown function: ") + symbol);
      }
    } else if (x->type() == NODE_LIST) { // (proc exp*)
      std::vector<std::shared_ptr<Node>> exps;
      for (auto &exp : *static_cast<ListNode *>(&*x)->children()) {
        exps.push_back(eval(exp));
      }

      std::function<std::shared_ptr<Node>(std::vector<std::shared_ptr<Node>> &)>
          glambda = [](std::vector<std::shared_ptr<Node>> &exp) {
            return std::shared_ptr<Node>(new StringNode("HAHAH"));
          };

      auto proc = exps[0];
      exps.erase(exps.begin());
      return static_cast<FunctionNode *>(&*proc)->call(exps);
      //            exps = [self.eval(exp, env) for exp in x]
      //            proc = exps.pop(0)
      //            return proc(*exps)
    } else {
      return x;
    }
  }

  std::shared_ptr<Node> run(std::string sexp) {
    return this->eval(parse(sexp));
  }
};

} // namespace tinylisp
} // namespace akaza