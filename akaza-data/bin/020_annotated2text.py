import glob
import multiprocessing as mp
import os
import pathlib
import sys
import time

from akaza_data_utils import get_sig, mkdir_p
from akaza_data_utils.merge_terms import merge_terms, load_skk_dict


def process(words):
    for word in words:
        m = word.split('/')
        if len(m) == 3:
            kanji, hinshi, yomi = m
            yield kanji, hinshi, yomi
        else:
            yield word, None, word


def annotated2text(fname, dest, skkdict, merged):
    with open(fname, 'r') as rfp, \
            open(dest, 'w') as wfp:
        for line in rfp:
            words = line.rstrip().split(' ')
            wfp.write(
                ' '.join(
                    ['/'.join(got) for got in merge_terms([x for x in process(words)], skkdict, merged)]
                ) + "\n")


def worker(chunk, skkdict):
    finished = 0
    merged = set()
    t0 = time.time()
    for fname in chunk:
        dest = fname.replace('work/stats-kytea/annotated/', 'work/stats-kytea/text/')
        pathlib.Path(dest).parent.mkdir(parents=True, exist_ok=True)
        print(
            f"[{os.getpid()}] [{sys.argv[0]}] {fname} -> {dest} ({finished}/{len(chunk)})"
            f" elapsed: {time.time() - t0}")
        annotated2text(fname, dest, skkdict, merged)
        finished += 1
    return merged


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()

    files = glob.glob('work/stats-kytea/annotated/*/wiki_*')
    chunks = split(files, numprocs)

    result_pool = []
    pool = mp.Pool(numprocs)

    print(f"Loading skk dict")
    skkdict = load_skk_dict()
    # print(skkdict)

    for chunk in chunks:
        result_pool.append(pool.apply_async(worker, args=(chunk, skkdict)))

    merged = set()
    while len(result_pool) > 0:
        print(f"Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                got = r.get()
                merged.update(got)
                result_pool.remove(r)
        time.sleep(1)

    sig = get_sig()
    mkdir_p('work/dump')
    with open(f'work/dump/{sig}-merged-annotated2text.txt', 'w') as wfp:
        for row in sorted(merged):
            wfp.write(f"{row}\n")


if __name__ == '__main__':
    main()
