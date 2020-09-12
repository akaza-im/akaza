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
