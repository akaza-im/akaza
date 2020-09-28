import datetime
import pathlib
import subprocess
import shutil
from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi
from typing import Set, Tuple


def get_sig():
    hash = subprocess.run(["git", "rev-parse", "--short", 'HEAD'], capture_output=True).stdout.decode(
        'utf-8').rstrip()
    sig = datetime.datetime.now().strftime('%Y%m%d-%H%M') + "-" + hash
    return sig


def mkdir_p(path: str):
    pathlib.Path(path).mkdir(exist_ok=True, parents=True)


def copy_snapshot(path: str):
    sig = get_sig()
    name = pathlib.Path(path).name
    pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
    shutil.copy(path, f'work/dump/{sig}-{name}')


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


def merge_terms(tuples, skkdict, merged: Set[str]):
    i = 0
    while i < len(tuples):
        if i + 1 < len(tuples):  # 次の単語がある
            kanji = tuples[i][0] + tuples[i + 1][0]
            kana = tuples[i][2] + tuples[i + 1][2]
            if kana in skkdict and kanji in skkdict[kana]:
                merged.add(f"{kana} -> {kanji} {tuples[i][1]}/{tuples[i + 1][1]}")
                # print(f"Merged: {kanji}/{kana}")
                yield kanji, kana
                i += 2
                continue

        yield tuples[i][0], tuples[i][2]
        i += 1
