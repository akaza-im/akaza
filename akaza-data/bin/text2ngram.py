import glob
import json
import multiprocessing as mp
import os
import pathlib
import sys
import time
# Usage: $0 wfreq
from typing import Set

import psutil

# jawiki.vocab と dat/*/* を元に、jawiki.1gram.json と jawiki.2gram.json を構築する。

vocabfname = sys.argv[1]

def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


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
            json.dump(self.d, fp, ensure_ascii=False, indent=1)


def count_ngram(fname: str, bigram):
    with open(fname) as rfp:
        for line in rfp:
            words = line.rstrip().split(' ')
            for i in range(0, len(words) - 1):
                bigram.register(words[i], words[i + 1])


def worker(vocab, chunk):
    finished = 0

    process = psutil.Process(os.getpid())
    t0 = time.time()

    for fname in chunk:
        print(f"[{os.getpid()}] {fname} ({finished}/{len(chunk)})"
              f"({process.memory_info().rss / 1024 / 1024} MB): {time.time() - t0}.")
        bigram = BiGram(vocab)
        count_ngram(fname, bigram)
        ofname = fname.replace('work/text/', 'work/2gram/') + ".2gram.json"
        pathlib.Path(ofname).parent.mkdir(parents=True, exist_ok=True)
        bigram.dump(ofname)
        finished += 1
    return 1


def main():
    t0 = time.time()

    vocab = set(read_vocab())
    assert 'で/で' in vocab

    numprocs = mp.cpu_count()
    pool = mp.Pool(numprocs)

    files = glob.glob('work/text/*/*')
    chunks = split(files, numprocs)

    result_pool = []

    for chunk in chunks:
        result_pool.append(pool.apply_async(worker, args=(vocab, chunk,)))

    print(f"Proceeded all files: {time.time() - t0}")

    while len(result_pool) > 0:
        print(f"Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                r.get()
                result_pool.remove(r)
        time.sleep(1)

    print(f"Finished: {time.time() - t0}")


if __name__ == '__main__':
    main()
