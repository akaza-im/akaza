from akaza_data.systemlm_loader import BinaryDict
from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi


def load_skk_file_dict(path: str, encoding: str = 'utf-8') -> BinaryDict:
    # TODO: cache したほうが良さそう。
    ari, nasi = parse_skkdict(path, encoding)
    merged = merge_skkdict([
        nasi,
        ari2nasi(ari)
    ])
    print(merged)

    d = BinaryDict()
    t = []
    for k, v in merged.items():
        t.append((k, '/'.join(v).encode('utf-8')))
    d.build(t)

    return d
