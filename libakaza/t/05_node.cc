#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

/*

def test_surface():
    e = TinyLisp()
    node = Node(word='(. "a" "b")', yomi='たしざんてすと', start_pos=0)
    assert node.surface(e) == "ab"

 */

void test_surface() {
    auto lisp = akaza::tinylisp::TinyLisp();
    auto node = akaza::Node(0, "たしざんてすと", R"((. "a" "b"))");
    ok(node.surface(lisp) == "ab");
}


int main() {
    test_surface();

    done_testing();
}