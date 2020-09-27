import glob
import logging
import multiprocessing as mp
import os
import pathlib
import sys
import time
from typing import Set

import psutil

# jawiki.vocab と work/text/*/* を元に、work/ngram/ を構築する。
from akaza_data_utils import mkdir_p


def read_vocab():
    with open('work/jawiki.vocab', 'r') as fp:
        return [line.rstrip() for line in fp.readlines()]


class NGram:
    wfreq: Set[str]

    def __init__(self, vocab: Set[str]):
        self.d = {}
        self.vocab = vocab

    def register(self, words):
        for word in words:
            if word not in self.vocab:
                return

        self.d[words] = self.d.get(words, 0) + 1

    def __len__(self):
        return len(self.d)

    def dump(self, fname):
        pathlib.Path(fname).parent.mkdir(exist_ok=True, parents=True)
        with open(fname, 'w') as fp:
            for words in sorted(self.d.keys()):
                word = "\t".join(words)
                fp.write(f"""{word} {self.d[words]}\n""")


def worker(chunk):
    t0 = time.time()

    vocab = set(read_vocab())

    file_count = 0
    for fname in chunk:
        elapsed = time.time() - t0
        process = psutil.Process(os.getpid())
        dest = fname.replace('work/text/', 'work/ngram/')

        print(f"[{sys.argv[0]}] {fname} -> {dest} {file_count}/{len(chunk)} "
              f"({process.memory_info().rss / 1024 / 1024} MB) "
              f" elapsed={elapsed}"
              f" remains={elapsed / max(file_count, 1) * (len(chunk) - file_count)}")

        unigram = NGram(vocab)
        bigram = NGram(vocab)
        trigram = NGram(vocab)
        with open(fname) as rfp:
            for line in rfp:
                words = line.rstrip().split(' ')
                for i in range(0, len(words)):
                    unigram.register((words[i],))
                    if i + 1 < len(words):
                        bigram.register((words[i], words[i + 1]))
                    if i + 2 < len(words):
                        trigram.register((words[i], words[i + 1], words[i + 2]))
            unigram.dump(f'{dest}.1gram.txt')
            bigram.dump(f'{dest}.2gram.txt')
            trigram.dump(f'{dest}.3gram.txt')

        file_count += 1

    print(f"Proceeded all files: {time.time() - t0}")

    # trigram.dump('work/jawiki.3gram.json', cutoff=TRIGRAM_CUTOFF)

    print(f"Finished: {time.time() - t0}")


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()

    files = glob.glob('work/text/*/*')
    chunks = split(files, numprocs)

    result_pool = []
    pool = mp.Pool(numprocs)

    for chunk in chunks:
        result_pool.append(pool.apply_async(worker, args=(chunk,)))

    while len(result_pool) > 0:
        print(f"Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                r.get()
                result_pool.remove(r)
        time.sleep(1)


if __name__ == '__main__':
    logging.basicConfig(level=logging.DEBUG)
    main()
