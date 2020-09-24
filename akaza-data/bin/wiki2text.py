import logging
import os
import pathlib
import sys
from Mykytea import Mykytea
import re

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


def main():
    count = 0
    total = len(sys.argv[1:])
    for ifile in sys.argv[1:]:
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
                line = re.sub('<nowiki>(.*?)</nowiki>', r'\1',  line)

                if len(line) == 0:
                    continue

                wfp.write(' '.join([x[0] + '/' + x[1] for x in parse_line_simple(line)]) + "\n")
        count += 1


if __name__ == '__main__':
    logging.basicConfig(level=logging.DEBUG)
    main()
