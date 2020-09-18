from akaza.node import Node


def test_node():
    assert [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ] == [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ]


def test_node_hash():
    assert hash(Node(word='ひょい', yomi='ひょい', start_pos=0))
