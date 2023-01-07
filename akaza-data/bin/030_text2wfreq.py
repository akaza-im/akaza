import multiprocessing as mp
import os
import time
import glob
import sys

from akaza_data_utils import copy_snapshot


def text2wfreq(fname, wfreq):
    with open(fname, 'r') as fp:
        for line in fp:
            words = line.rstrip().split(' ')
            for word in words:
                wfreq[word] = wfreq.get(word, 0) + 1


def worker(chunk):
    wfreq = {}
    finished = 0
    t0 = time.time()
    for fname in chunk:
        print(f"[{os.getpid()}] [{sys.argv[0]}] {fname} ({finished}/{len(chunk)})"
              f" elapsed={time.time()-t0}")
        text2wfreq(fname, wfreq)
        finished += 1
    return wfreq


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()

    files = glob.glob('work/stats-kytea/text/*/wiki_*')
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

    with open('work/stats-kytea/jawiki.wfreq', 'w') as wfp:
        for key in sorted(merged_wfreq.keys()):
            count = merged_wfreq[key]
            if key != '__EOS__/__EOS__':
                if len(key.split('/')) != 2:
                    continue
                if len(key.split('/')[0]) == 0:
                    # `/"` のような謎のエントリを除外
                    continue
                if key.split('/')[0] == '\u3000':
                    # `　/くうはく` のような謎のエントリを除外
                    continue
                if '/' not in key:
                    continue
                if key.endswith('/UNK'):
                    continue
                if '/' == key:
                    continue
            wfp.write(f"{key} {count}\n")

    copy_snapshot('work/stats-kytea/jawiki.wfreq')


if __name__ == '__main__':
    main()
