import sys
import glob
import re
import os
import psutil
import json
import time

# Usage: $0 wfreq
from typing import Set

vocabfname = sys.argv[1]

SPACES = re.compile(r'\s+')


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
            json.dump(self.d, fp, ensure_ascii=False)


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
            json.dump(self.d, fp, ensure_ascii=False)


def main():
    t0 = time.time()

    vocab = set(read_vocab())

    unigram = UniGram(vocab)
    bigram = BiGram(vocab)

    all_files = len(glob.glob('dat/*/*'))
    file_count = 0
    for fname in glob.glob('dat/*/*'):
        with open(fname) as rfp:
            process = psutil.Process(os.getpid())
            print(f"{fname} {file_count}/{all_files} "
                  f"({process.memory_info().rss / 1024 / 1024} MB): {time.time() - t0}."
                  f" unigram: {len(unigram)}. bigram: {len(bigram)}")
            for line in rfp:
                words = SPACES.split(line.rstrip())
                for i in range(0, len(words) - 1):
                    unigram.register(words[i])
                    bigram.register(words[i], words[i + 1])
                unigram.register(words[-1])
        file_count += 1

    print(f"Proceeded all files: {time.time() - t0}")

    unigram.dump('jawiki.1gram.json')
    bigram.dump('jawiki.2gram.json')

    print(f"Finished: {time.time() - t0}")

    # 2gram なら、全部オンメモリで5分ぐらいで終る。


main()
