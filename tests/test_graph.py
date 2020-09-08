from comb.combromkan import to_hiragana
import marisa_trie
from comb.system_dict import SystemDict
from comb.graph import lookup, graph_construct, viterbi

unigram_score = marisa_trie.RecordTrie('@f')
unigram_score.load('model/jawiki.1gram')

bigram_score = marisa_trie.RecordTrie('@f')
bigram_score.load('model/jawiki.2gram')

system_dict = SystemDict()


def test_wnn():
    src = 'わたしのなまえはなかのです'
    expected = '私の名前は中野です'

    ht = dict(lookup(src, system_dict))
    graph = graph_construct(src, ht, unigram_score, bigram_score)

    nodes = viterbi(graph)
    # print(graph)
    got = ''.join([x.word for x in nodes if not x.is_eos()])

    assert got == expected
