#include "../include/akaza.h"
#include "../picotest/picotest.h"
#include "../picotest/picotest.c"

/*

def test_surface():
    e = TinyLisp()
    node = Node(word='(. "a" "b")', yomi='たしざんてすと', start_pos=0)
    assert node.surface(e) == "ab"

 */

static void test_surface() {
    auto lisp = akaza::tinylisp::TinyLisp();
    auto node = akaza::Node(0, L"たしざんてすと", LR"((. "a" "b"))");
    ok(node.surface(lisp) == L"ab");
}

static void test_eq() {
    auto a = akaza::Node(0, L"あ", L"あ");
    auto b = akaza::Node(0, L"い", L"い");

    ok(a == a);
    ok(b == b);
    ok(b != a);
}


int main() {
    test_surface();
    test_eq();

    done_testing();
}