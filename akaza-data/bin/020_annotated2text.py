import glob
import multiprocessing as mp
import os
import pathlib
import sys
import time
from typing import Set, Tuple

from akaza_data_utils import get_sig
from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi


def load_skk_dict():
    dictionary_sources = [
        # 先の方が優先される
        ('skk-dev-dict/SKK-JISYO.L', 'euc-jp'),
    ]
    dicts = []

    for path, encoding in dictionary_sources:
        ari, nasi = parse_skkdict(path, encoding)
        dicts.append(nasi)
        dicts.append(ari2nasi(ari))

    return merge_skkdict(dicts)


def process2(tuples, skkdict, merged: Set[Tuple[str]]):
    i = 0
    while i < len(tuples):
        if i + 1 < len(tuples):  # 次の単語がある
            kanji = tuples[i][0] + tuples[i + 1][0]
            kana = tuples[i][2] + tuples[i + 1][2]
            if kana in skkdict and kanji in skkdict[kana] and (
                    tuples[i][1] == '接頭辞' or tuples[i][2] == '語尾'
            ):
                merged.add( \
                    f"{tuples[i][0]}/{tuples[i][1]}{tuples[i][2]} ->" + \
                    f" {tuples[i + 1][0]}/{tuples[i + 1][1]}/{tuples[i + 1][2]}")
                # print(f"Merged: {kanji}/{kana}")
                yield kanji, kana
                i += 2
                continue

        yield tuples[i][0], tuples[i][2]

        i += 1


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
                ' '.join(['/'.join(got) for got in process2([x for x in process(words)], skkdict, merged)]))


def worker(chunk, skkdict):
    finished = 0
    merged = set()
    for fname in chunk:
        dest = fname.replace('work/annotated/', 'work/text/')
        pathlib.Path(dest).parent.mkdir(parents=True, exist_ok=True)
        print(f"[{os.getpid()}] [{sys.argv[0]}] {fname} -> {dest} ({finished}/{len(chunk)})")
        annotated2text(fname, dest, skkdict, merged)
        finished += 1
    return merged


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()

    files = glob.glob('work/annotated/*/wiki_*')
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
    with open(f'work/dump/{sig}-merged-annotated2text.txt', 'w') as wfp:
        for row in sorted(merged):
            wfp.write(f"{row}\n")


if __name__ == '__main__':
    main()
