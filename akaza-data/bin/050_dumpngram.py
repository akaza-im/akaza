import sys
import glob
import re
import os
import psutil
import json
import time

# jawiki.vocab と dat/*/* を元に、jawiki.1gram.json と jawiki.2gram.json を構築する。

# Usage: $0 wfreq
from typing import Set

vocabfname = sys.argv[1]

SPACES = re.compile(r'\s+')
BIGRAM_CUTOFF = 3


def read_vocab():
    with open(vocabfname, 'r') as fp:
        return [line.rstrip() for line in fp.readlines()]


class UniGram:
    def __init__(self, vocab: Set[str]):
        self.d = {}
        self.vocab = vocab

    def register(self, word):
        if word not in self.vocab:
            return

        if word not in self.d:
            self.d[word] = 0
        self.d[word] += 1

    def __len__(self):
        return len(self.d)

    def dump(self, fname):
        with open(fname, 'w') as fp:
            json.dump(self.d, fp, ensure_ascii=False, indent=1)


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


class TriGram:
    wfreq: Set[str]

    def __init__(self, vocab: Set[str]):
        self.d = {}
        self.vocab = vocab

    def register(self, word1, word2, word3):
        if word1 not in self.vocab or word2 not in self.vocab or word3 not in self.vocab:
            return

        if word1 not in self.d:
            self.d[word1] = {}
        if word2 not in self.d[word1]:
            self.d[word1][word2] = {}
        if word3 not in self.d[word1][word2]:
            self.d[word1][word2][word3] = 0
        self.d[word1][word2][word3] += 1

    def __len__(self):
        return len(self.d)

    def dump(self, fname):
        with open(fname, 'w') as fp:
            removelist = []
            for word1 in self.d:
                for word2 in self.d[word1]:
                    for word3, cnt in self.d[word1][word2].items():
                        if cnt <= BIGRAM_CUTOFF:
                            removelist.append((word1, word2, word3))
            for word1, word2, word3 in removelist:
                del self.d[word1][word2][word3]
            json.dump(self.d, fp, ensure_ascii=False, indent=1)


def main():
    t0 = time.time()

    vocab = set(read_vocab())

    unigram = UniGram(vocab)
    bigram = BiGram(vocab)
    trigram = TriGram(vocab)

    all_files = len(glob.glob('work/text/*/*'))
    file_count = 0
    for fname in glob.glob('work/text/*/*'):
        with open(fname) as rfp:
            process = psutil.Process(os.getpid())
            print(f"[{sys.argv[0]}] {fname} {file_count}/{all_files} "
                  f"({process.memory_info().rss / 1024 / 1024} MB): {time.time() - t0}."
                  f" unigram: {len(unigram)}. bigram: {len(bigram)}. trigram: {len(trigram)}")
            for line in rfp:
                words = SPACES.split(line.rstrip())
                for i in range(0, len(words)):
                    unigram.register(words[i])
                    if i + 1 < len(words):
                        bigram.register(words[i], words[i + 1])
                    if i + 2 < len(words) - 1:
                        trigram.register(words[i], words[i + 1], words[i + 2])
        file_count += 1

    print(f"Proceeded all files: {time.time() - t0}")

    unigram.dump('work/jawiki.1gram.json')
    bigram.dump('work/jawiki.2gram.json')
    trigram.dump('work/jawiki.3gram.json')

    print(f"Finished: {time.time() - t0}")

    # 2gram なら、全部オンメモリで5分ぐらいで終る。


if __name__ == '__main__':
    main()
