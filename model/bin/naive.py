import sys
import re

import marisa_trie

# とりあえずでつくった、1gram のデータをダスやつ。

arpafname = sys.argv[1]

SPACES = re.compile(r'\s+')

# unigram かいていく
retval = []
with open(arpafname, 'r') as fp:
    for line in fp:
        if line == "\\1-grams:\n":
            break

    for line in fp:
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


