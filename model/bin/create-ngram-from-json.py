import json
import math
import re
import time

import marisa_trie

# とりあえずでつくった、1gram のデータをダスやつ。

SPACES = re.compile(r'\s+')

BIGRAM_CUTOFF = 1


def write_1gram():
    # unigram かいていく
    retval = []
    with open('jawiki.1gram.json') as fp:
        data = json.load(fp)

        total = sum(data.values())

        for word, count in data.items():
            score = math.log10(count / total)
            retval.append((word, (float(score),),))

    trie = marisa_trie.RecordTrie('<f', retval)
    fname = 'jawiki.1gram'
    print(f"writing {fname}. size={len(retval)}")
    trie.save(fname)


def write_2gram():
    # bigram かいていく
    retval = []
    with open('jawiki.2gram.json', 'r') as fp:
        data = json.load(fp)
        for word1, word2data in data.items():
            total = sum(word2data.values())
            for word2, count in word2data.items():
                if count <= BIGRAM_CUTOFF:
                    continue
                score = math.log10(count / total)
                retval.append((f"{word1}\t{word2}", (float(score),),))

    trie = marisa_trie.RecordTrie('<f', retval)
    fname = 'jawiki.2gram'
    print(f"writing {fname}. size={len(retval)}")
    trie.save(fname)


def main():
    t0 = time.time()
    write_1gram()
    write_2gram()
    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    main()
