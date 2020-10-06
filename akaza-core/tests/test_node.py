import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.node import Node
from akaza_data.systemlm_loader import TinyLisp


def test_node():
    assert [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ] == [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ]


def test_surface():
    e = TinyLisp()
    node = Node(word='(. "a" "b")', yomi='たしざんてすと', start_pos=0)
    assert node.surface(e) == "ab"
