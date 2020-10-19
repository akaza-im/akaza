import sys
import pathlib
import pytest

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from pyakaza.bind import BinaryDict, SystemUnigramLM, SystemBigramLM

ulm = SystemUnigramLM()
ulm.load('../akaza-data/data/lm_v2_1gram.trie')

blm = SystemBigramLM()
blm.load('../akaza-data/data/lm_v2_2gram.trie')


def test_foobar():
    assert 'find_unigram' in dir(SystemUnigramLM)


def test_unigram2():
    assert ulm.find_unigram('愛/あい')[1] != ulm.find_unigram('安威/あい')[1]


def test_unigram_siin():
    assert ulm.find_unigram('子音/しいん')[1] != ulm.find_unigram('試飲/しいん')[1]


def test_find_unigram():
    id, score = ulm.find_unigram('私/わたし')
    print([id, score])
    assert id > 0
    assert score < 0


def test_find_bigram():
    id_watasi, _ = ulm.find_unigram('私/わたし')
    id_ja, _ = ulm.find_unigram('じゃ/じゃ')
    score = blm.find_bigram(id_watasi, id_ja)
    assert score < 0


@pytest.mark.skip(reason="slow test")
def test_find_unigram_test_all():
    path = 'work/jawiki.merged-1gram.txt'
    if not os.path.exists(path):
        return
    with open(path, 'r') as fp:
        for line in fp:
            word, txt_score = line.rstrip().split(' ')
            id, trie_score = ulm.find_unigram(word)
            print(f"word='{word}' id={id} trie_score={trie_score} txt_score={txt_score}")
            assert abs(trie_score - float(txt_score)) < 0.000001


@pytest.mark.skip(reason="slow test")
def test_find_bigram_test_all():
    path = 'work/jawiki.merged-2gram.txt'
    if not os.path.exists(path):
        return
    with open(path, 'r') as fp:
        for line in fp:
            words, txt_score = line.rstrip().split(' ')
            word1, word2 = words.split("\t")
            id1, _ = ulm.find_unigram(word1)
            id2, _ = ulm.find_unigram(word2)
            trie_score = blm.find_bigram(id1, id2)
            print(f"word='{word1}-{word2}' id={id1}-{id2} trie_score={trie_score} txt_score={txt_score}")
            assert abs(trie_score - float(txt_score)) < 0.000001


if __name__ == '__main__':
    # test_find_unigram_test_all()
    test_find_bigram_test_all()
