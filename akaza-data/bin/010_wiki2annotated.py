import logging
import os
import pathlib
import sys
from Mykytea import Mykytea
import re
import multiprocessing as mp
import glob
import time

kytea = Mykytea('-model work/kytea/train.mod')

HIRAGANA_BLOCK = r'\u3041-\u309F'
HIRAGANA_PATTERN = re.compile(r'^[' + HIRAGANA_BLOCK + ']+$')

# 上級個人情報保護士（じょうきゅうこじんじょうほうほごし）は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
# → 上級個人情報保護士は、財団法人全日本情報学習振興協会が設けている民間資格の称号。
YOMIGANA_PATTERN = re.compile(r'[（\(][' + HIRAGANA_BLOCK + r'、]+[）)]')


def is_hiragana(s):
    if HIRAGANA_PATTERN.match(s):
        return True
    else:
        return False


def cleanup(s):
    return re.sub(YOMIGANA_PATTERN, '', s)


def parse_line(line):
    line = cleanup(line)

    words = kytea.getTags(line)

    yield '__BOS__', '__BOS__'

    for word in words:
        kanji = word.surface
        hinshi = word.tag[0][0][0]
        yomi = word.tag[1][0][0]
        if kanji == ' ':
            # 空白のみのアイテムは無視してよさそう。
            continue
        yield kanji, hinshi, yomi

    yield '__EOS__', '__EOS__'  # EOS


def process_files(files):
    count = 0
    total = len(files)
    t0 = time.time()
    for ifile in files:
        ofile = ifile.replace('work/extracted/', 'work/annotated/')

        pathlib.Path(ofile).parent.mkdir(parents=True, exist_ok=True)

        elapsed = time.time() - t0
        logging.info(
            f"[{os.getpid()}] {ifile} -> {ofile} ({count}/{total}) elapsed={elapsed}"
            f" expected={(total * elapsed) / max(count, 1)}")
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

                wfp.write(' '.join(['/'.join(x) for x in parse_line(line)]) + "\n")
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
        print(f"[{sys.argv[0]}] Remains: {len(result_pool)}")
        for r in result_pool:
            if r.ready():
                r.get()
                result_pool.remove(r)
        time.sleep(1)

    print(f"Finished: {time.time() - t0}")


if __name__ == '__main__':
    logging.basicConfig(level=logging.DEBUG)
    main()
