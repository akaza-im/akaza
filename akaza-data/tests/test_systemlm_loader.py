import os
import sys

sys.path.insert(0, '.')

from akaza_data.systemlm_loader import SystemLM

lm = SystemLM()
lm.load('akaza_data/data/lm_v2_1gram.trie', 'akaza_data/data/lm_v2_2gram.trie')


def test_foobar():
    assert 'find_unigram' in dir(SystemLM)


def test_find_unigram():
    id, score = lm.find_unigram('私/わたし')
    print([id, score])
    assert id > 0
    assert score < 0


def test_find_bigram():
    id_watasi, _ = lm.find_unigram('私/わたし')
    id_ja, _ = lm.find_unigram('じゃ/じゃ')
    score = lm.find_bigram(id_watasi, id_ja)
    assert score < 0


def test_find_unigram_test_all():
    path = 'work/jawiki.merged-1gram.txt'
    if not os.path.exists(path):
        return
    with open(path, 'r') as fp:
        for line in fp:
            word, txt_score = line.rstrip().split(' ')
            id, trie_score = lm.find_unigram(word)
            print(f"word='{word}' id={id} trie_score={trie_score} txt_score={txt_score}")
            assert abs(trie_score - float(txt_score)) < 0.000001


def test_find_bigram_test_all():
    path = 'work/jawiki.merged-2gram.txt'
    if not os.path.exists(path):
        return
    with open(path, 'r') as fp:
        for line in fp:
            words, txt_score = line.rstrip().split(' ')
            word1, word2 = words.split("\t")
            id1, _ = lm.find_unigram(word1)
            id2, _ = lm.find_unigram(word2)
            trie_score = lm.find_bigram(id1, id2)
            print(f"word='{word1}-{word2}' id={id1}-{id2} trie_score={trie_score} txt_score={txt_score}")
            assert abs(trie_score - float(txt_score)) < 0.000001

if __name__ == '__main__':
    # test_find_unigram_test_all()
    test_find_bigram_test_all()
