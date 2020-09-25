import multiprocessing as mp
import os
import time
import glob
import sys
import pathlib


def process(words):
    for word in words:
        m = word.split('/')
        if len(m) == 3:
            kanji, hinshi, yomi = m
            yield kanji, yomi
        else:
            yield word, word


def annotated2text(fname, dest):
    with open(fname, 'r') as rfp, \
            open(dest, 'w') as wfp:
        for line in rfp:
            words = line.rstrip().split(' ')
            wfp.write(' '.join(['/'.join(got) for got in process(words)]))


def worker(chunk):
    wfreq = {}
    finished = 0
    for fname in chunk:
        dest = fname.replace('work/annotated/', 'work/text/')
        pathlib.Path(dest).parent.mkdir(parents=True, exist_ok=True)
        print(f"[{os.getpid()}] [{sys.argv[0]}] {fname} -> {dest} ({finished}/{len(chunk)})")
        annotated2text(fname, dest)
        finished += 1
    return wfreq


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()

    files = glob.glob('work/annotated/*/wiki_*')
    chunks = split(files, numprocs)

    result_pool = []
    pool = mp.Pool(numprocs)

    for chunk in chunks:
        result_pool.append(pool.apply_async(worker, args=(chunk,)))

    merged_wfreq = {}
    while len(result_pool) > 0:
        print(f"Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                wfreq_part = r.get()
                for k, v in wfreq_part.items():
                    merged_wfreq[k] = merged_wfreq.get(k, 0) + v
                result_pool.remove(r)
        time.sleep(0.1)


if __name__ == '__main__':
    main()
