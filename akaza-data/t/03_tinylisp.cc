#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

#include "../src/tinylisp.h"
#include <iostream>
#include <typeinfo>

using namespace akaza::tinylisp;

int main() {
    akaza::tinylisp::TinyLisp tinylisp;

    {
        std::shared_ptr<Node> got = tinylisp.parse("(a \"abc\")");

        ok(got->type() == NODE_LIST);
        ok(static_cast<ListNode*>(&*got)->children()->size() == 2);

        auto a = static_cast<ListNode*>(&*got)->children()->at(0);
        ok(a->type() == NODE_SYMBOL);
        ok(static_cast<SymbolNode*>(&*a)->symbol() == "a");

        auto abc = static_cast<ListNode*>(&*got)->children()->at(1);
        ok(abc->type() == NODE_STRING);
        ok(static_cast<StringNode*>(&*abc)->str() == "abc");
    }

    {
        std::shared_ptr<Node> got = tinylisp.run("(strftime (current-datetime) \"%Y-%m-%d\")");
        ok(got->type() == NODE_STRING);
        std::string got_str = static_cast<StringNode*>(&*got)->str();
        note("%s", got_str.c_str());
    }

    done_testing();
}
