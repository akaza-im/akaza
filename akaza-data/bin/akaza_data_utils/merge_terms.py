from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi
from typing import Set


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


IDX_KANJI = 0
IDX_HINSHI = 1
IDX_KANA = 2


def merge_terms(tuples, skkdict, merged: Set[str]):
    i = 0
    while i < len(tuples):
        if i + 1 < len(tuples):  # 次の単語がある
            # できるかぎり分節単位にしていきたいので、語尾/助詞/助動詞 などは連結する。
            if tuples[i][IDX_KANA] != 'UNK' \
                    and tuples[i][IDX_HINSHI] == '動詞' \
                    and tuples[i + 1][IDX_KANA] != 'UNK' \
                    and tuples[i + 1][IDX_HINSHI] in ('語尾', '助詞', '助動詞'):
                kanji = tuples[i][IDX_KANJI]
                kana = tuples[i][IDX_KANA]
                i += 1
                hinshis = [tuples[i][IDX_HINSHI]]
                while i < len(tuples) \
                        and tuples[i][IDX_HINSHI] in ('語尾', '助詞', '助動詞') \
                        and tuples[i][IDX_KANA] != 'UNK':
                    kanji += tuples[i][IDX_KANJI]
                    kana += tuples[i][IDX_KANA]
                    hinshis.append(tuples[i][IDX_HINSHI])
                    i += 1
                merged.add(f"{kana} -> {kanji} {'/'.join(hinshis)}")
                yield kanji, kana
                continue

            # 単純に次の単語と連結したものが登録されている場合
            # e.g. 小/接頭辞/しょう 学校/名詞/がっこう
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
