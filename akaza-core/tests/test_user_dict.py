from tempfile import TemporaryDirectory

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel


def test_read():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='熟語', yomi='じゅくご')])
    assert d.unigram == {'単語/たんご': 2, '熟語/じゅくご': 1}
    assert d.total == 3


def test_read2():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='それは', yomi='それは'),
        Node(start_pos=3, word='私', yomi='わたし'),
        Node(start_pos=4, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='です', yomi='です'),
    ])

    assert d.unigram == {'それは/それは': 1, 'だよ/だよ': 2, '私/わたし': 3, 'です/です': 1}
    assert d.total == 7

    assert d.bigram == {'それは/それは\t私/わたし': 1, '私/わたし\tだよ/だよ': 2, '私/わたし\tです/です': 1}
    assert d.bigram_total == {'それは/それは': 1, '私/わたし': 3}
