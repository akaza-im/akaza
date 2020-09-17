import json
import math
import re
import time

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


def write_model():
    # bigram かいていく
    retval = []

    print('# 1gram')
    with open('jawiki.1gram.json') as fp:
        data = json.load(fp)

        total = sum(data.values())

        for word in sorted(data.keys()):
            count = data[word]
            score = math.log10(count / total)

            retval.append((word, (float(score),),))

    print(f"1gram. size={len(retval)}")
    unigram_size = len(retval)

    print('# 2gram')
    with open('jawiki.2gram.json', 'r') as fp:
        data = json.load(fp)

        for word1, word2data in data.items():
            total = sum(word2data.values())

            for word2, count in word2data.items():
                if count <= BIGRAM_CUTOFF:
                    continue

                score = math.log10(count / total)
                retval.append((f"{word1}\t{word2}", (float(score),),))

    print(f"2gram. size={len(retval) - unigram_size}")

    trie = marisa_trie.RecordTrie('<f', retval)
    fname = 'akaza_data/data/system_language_model.trie'
    print(f"writing {fname}. size={len(retval)}")
    trie.save(fname)


def main():
    t0 = time.time()
    write_model()
    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    main()
