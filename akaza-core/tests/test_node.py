import pathlib
import sys

from akaza import tinylisp

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.node import Node


def test_node():
    assert [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ] == [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ]


def test_node_hash():
    assert hash(Node(word='ひょい', yomi='ひょい', start_pos=0))


def test_surface():
    e = tinylisp.Evaluator()
    node = Node(word='(+ 1 2)', yomi='たしざんてすと', start_pos=0)
    assert node.surface(e) == 3
