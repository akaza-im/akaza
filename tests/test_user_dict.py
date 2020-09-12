from tempfile import NamedTemporaryFile, TemporaryDirectory

from comb.node import Node
from comb.user_dict import UserDict
import marisa_trie

unigram_score = marisa_trie.RecordTrie('@f')
unigram_score.load('model/jawiki.1gram')

bigram_score = marisa_trie.RecordTrie('@f')
bigram_score.load('model/jawiki.2gram')


def test_read():
    tmpdir = TemporaryDirectory()
    d = UserDict(tmpdir.name + "/foobar.dict")
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='熟語', yomi='じゅくご')])
    assert d.unigram == {'単語/たんご': 2, '熟語/じゅくご': 1}
    assert d.total == 3


def test_read2():
    tmpdir = TemporaryDirectory()
    d = UserDict(tmpdir.name + "/foobar.dict")
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
