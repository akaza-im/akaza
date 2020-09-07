import sys
import re

import marisa_trie

# とりあえずでつくった、1gram のデータをダスやつ。

arpafname = sys.argv[1]

SPACES = re.compile(r'\s+')


def write_1gram():
    # unigram かいていく
    retval = []
    with open(arpafname, 'r') as fp:
        for line in fp:
            if line == "\\1-grams:\n":
                break

        for line in fp:
            if line == "\\2-grams:\n":
                break
            # process it
            line = line.lstrip()
            m = SPACES.split(line)
            if len(m) >= 2:
                score = m[0]
                word = m[1]
                retval.append((word, (float(score),),))
            else:
                break

    trie = marisa_trie.RecordTrie('<f', retval)
    fname = 'jawiki.1gram'
    print(f"writing {fname}. size={len(retval)}")
    trie.save(fname)


def write_2gram():
    # bigram かいていく
    retval = []
    with open(arpafname, 'r') as fp:
        for line in fp:
            if line == "\\2-grams:\n":
                break

        for line in fp:
            if line == "\\3-grams:\n":
                break
            # process it
            line = line.lstrip()
            m = SPACES.split(line)
            if len(m) >= 2:
                score = m[0]
                word1 = m[1]
                word2 = m[2]
                retval.append((f"{word1}\t{word2}", (float(score),),))
            else:
                break

    trie = marisa_trie.RecordTrie('<f', retval)
    fname = 'jawiki.2gram'
    print(f"writing {fname}. size={len(retval)}")
    trie.save(fname)


def main():
    write_1gram()
    write_2gram()


if __name__ == '__main__':
    main()
