import json
import math
import re
import time
import sys

import marisa_trie

# jawiki.1gram.json/jawiki.2gram.json から言語モデルを出力する。

SPACES = re.compile(r'\s+')

BIGRAM_CUTOFF = 2


# 漢字/よみ → よみ/漢字 に変更する。
def reverse_word(word):
    m = word.split('/')
    if len(m) != 2:
        raise RuntimeError(f"---{word}---")
    kanji, yomi = m
    return f"{yomi}/{kanji}"


def build_1gram():
    retval = []
    with open('work/jawiki.1gram.json') as fp:
        data = json.load(fp)

        total = sum(data.values())

        for word in sorted(data.keys()):
            count = data[word]
            score = math.log10(count / total)

            retval.append((word, (float(score),),))

    return retval


def build_2gram():
    retval = []

    with open('work/jawiki.2gram.json', 'r') as fp:
        data = json.load(fp)

        for word1, word2data in data.items():
            total = sum(word2data.values())

            for word2, count in word2data.items():
                if count <= BIGRAM_CUTOFF:
                    continue

                score = math.log10(count / total)
                retval.append((f"{word1}\t{word2}", (float(score),),))

    return retval


def write_model():
    # bigram かいていく
    retval = []

    print('# 1gram')
    unigram = build_1gram()

    print(f"1gram. size={len(unigram)}")

    print('# 2gram')
    bigram = build_2gram()

    print(f"[{sys.argv[0]}] 2gram. size={len(bigram)}")

    trie = marisa_trie.RecordTrie('<f', unigram + bigram)
    fname = 'akaza_data/data/system_language_model.trie'
    print(f"writing {fname}.")
    trie.save(fname)


def main():
    t0 = time.time()
    write_model()
    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    main()
