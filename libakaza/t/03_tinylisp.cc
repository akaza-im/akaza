#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../include/tinylisp.h"
#include <iostream>
#include <typeinfo>

using namespace akaza::tinylisp;

int main() {
    akaza::tinylisp::TinyLisp tinylisp;

    {
        std::shared_ptr<Node> got = tinylisp.parse("(a \"abc\")");

        ok(got->type() == NODE_LIST);
        ok(dynamic_cast<ListNode*>(&*got)->children()->size() == 2);

        auto a = dynamic_cast<ListNode*>(&*got)->children()->at(0);
        ok(a->type() == NODE_SYMBOL);
        ok(dynamic_cast<SymbolNode*>(&*a)->symbol() == "a");

        auto abc = dynamic_cast<ListNode*>(&*got)->children()->at(1);
        ok(abc->type() == NODE_STRING);
        ok(dynamic_cast<StringNode*>(&*abc)->str() == "abc");
    }

    {
        std::shared_ptr<Node> got = tinylisp.run_node("(strftime (current-datetime) \"%Y-%m-%d\")");
        ok(got->type() == NODE_STRING);
        std::string got_str = dynamic_cast<StringNode*>(&*got)->str();
        note("%s", got_str.c_str());
    }

    {
        std::shared_ptr<Node> got = tinylisp.run_node(R"((. "a" "b"))");
        ok(got->type() == NODE_STRING);
        std::string got_str = dynamic_cast<StringNode*>(&*got)->str();
        ok(got_str == "ab");
    }

    done_testing();
}
