import sys
import glob
import os
import psutil
import json
import time

# jawiki.vocab と dat/*/* を元に、jawiki.2gram.json を構築する。

# Usage: $0 wfreq
from typing import Set

vocabfname = sys.argv[1]

BIGRAM_CUTOFF = 3


def read_vocab():
    with open(vocabfname, 'r') as fp:
        return [line.rstrip() for line in fp.readlines()]


class BiGram:
    wfreq: Set[str]

    def __init__(self, vocab: Set[str]):
        self.d = {}
        self.vocab = vocab

    def register(self, word1, word2):
        if word1 not in self.vocab or word2 not in self.vocab:
            return

        if word1 not in self.d:
            self.d[word1] = {}
        if word2 not in self.d[word1]:
            self.d[word1][word2] = 0
        self.d[word1][word2] += 1

    def __len__(self):
        return len(self.d)

    def dump(self, fname):
        with open(fname, 'w') as fp:
            removelist = []
            for word1 in self.d:
                for word2 in self.d[word1]:
                    if self.d[word1][word2] <= BIGRAM_CUTOFF:
                        removelist.append((word1, word2))
            for word1, word2 in removelist:
                del self.d[word1][word2]
            json.dump(self.d, fp, ensure_ascii=False, indent=1)


def main():
    t0 = time.time()

    vocab = set(read_vocab())

    bigram = BiGram(vocab)

    all_files = len(glob.glob('work/text/*/*'))
    file_count = 0
    for fname in glob.glob('work/text/*/*'):
        with open(fname) as rfp:
            process = psutil.Process(os.getpid())
            print(f"{fname} {file_count}/{all_files} "
                  f"({process.memory_info().rss / 1024 / 1024} MB): {time.time() - t0}."
                  f" bigram: {len(bigram)}")
            for line in rfp:
                words = line.rstrip().split(' ')
                for i in range(0, len(words) - 1):
                    bigram.register(words[i], words[i + 1])
        file_count += 1

    print(f"Proceeded all files: {time.time() - t0}")

    bigram.dump('work/jawiki.2gram.json')

    print(f"Finished: {time.time() - t0}")

    # 2gram なら、全部オンメモリで5分ぐらいで終る。


main()
