import logging
import os
import pathlib
import sys
from Mykytea import Mykytea
import re
import multiprocessing as mp
import glob
import time
import psutil

kytea = Mykytea('-model /usr/share/kytea/model.bin')

HIRAGANA_BLOCK = r'\u3041-\u309F'
HIRAGANA_PATTERN = re.compile(r'^[' + HIRAGANA_BLOCK + ']+$')


def is_hiragana(s):
    if HIRAGANA_PATTERN.match(s):
        return True
    else:
        return False


def parse_line(line):
    words = kytea.getTags(line)

    yield '<S>', '<S>'  # BOS

    # 連続する平仮名エントリーを、連結する。
    hiragana_queue = []

    for word in words:
        kanji = word.surface
        yomi = word.tag[1][0][0]
        if is_hiragana(kanji):
            hiragana_queue.append(kanji)
        else:
            if len(hiragana_queue) > 0:
                kana = ''.join(hiragana_queue)
                hiragana_queue = []
                yield kana, kana
            if kanji == ' ':
                # 空白のみのアイテムは無視してよさそう。
                continue
            yield kanji, yomi

    if len(hiragana_queue) > 0:
        kana = ''.join(hiragana_queue)
        yield kana, kana

    yield '</S>', '</S>'  # EOS


def parse_line_simple(line):
    words = kytea.getTags(line)

    yield '<S>', '<S>'  # BOS

    for word in words:
        kanji = word.surface
        yomi = word.tag[1][0][0]
        if kanji == ' ':
            # 空白のみのアイテムは無視してよさそう。
            continue
        yield kanji, yomi

    yield '</S>', '</S>'  # EOS


def process_files(files):
    count = 0
    total = len(files)
    for ifile in files:
        ofile = ifile.replace('work/extracted/', 'work/text/')

        pathlib.Path(ofile).parent.mkdir(parents=True, exist_ok=True)

        logging.info(f"[{os.getpid()}] {ifile} -> {ofile} ({count}/{total})")
        with open(ifile, 'r') as rfp, \
                open(ofile, 'w') as wfp:
            last_line_is_open_tag = False
            for line in rfp:
                # タグはじまりの行の次の行はスキップする。
                if last_line_is_open_tag:
                    last_line_is_open_tag = False
                    continue

                # タグ始まりの行をスキップする
                if line.startswith('<doc'):
                    last_line_is_open_tag = True
                    continue
                if line.startswith('<'):
                    continue

                line = line.rstrip()
                # strip <nowiki> tag.
                line = re.sub('<nowiki>(.*?)</nowiki>', r'\1', line)
                line = re.sub('<br>', r' ', line, flags=re.I)

                if len(line) == 0:
                    continue

                wfp.write(' '.join([x[0] + '/' + x[1] for x in parse_line_simple(line)]) + "\n")
        count += 1


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


def main():
    numprocs = mp.cpu_count()
    logging.info(f"numprocs={numprocs}")

    files = glob.glob('work/extracted/*/wiki_*')
    chunks = split(files, numprocs)
    pool = mp.Pool(numprocs)

    t0 = time.time()

    result_pool = []
    for chunk in chunks:
        result_pool.append(pool.apply_async(process_files, args=(chunk,)))

    while len(result_pool) > 0:
        print(f"Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                r.get()
                result_pool.remove(r)
        time.sleep(1)

    print(f"Finished: {time.time() - t0}")


if __name__ == '__main__':
    logging.basicConfig(level=logging.DEBUG)
    main()
