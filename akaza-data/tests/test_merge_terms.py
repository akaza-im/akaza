import sys
import pathlib
import pytest

path = str(pathlib.Path(__file__).parent.parent.joinpath('bin'))
sys.path.append(path)

from akaza_data_utils import merge_terms, load_skk_dict

skkdict = load_skk_dict()


@pytest.mark.parametrize('src, expected', [
    ('小/接頭辞/しょう 学校/名詞/がっこう', [('小学校', 'しょうがっこう')]),
    ('書/動詞/か く/語尾/く', [('書く', 'かく')]),
])
def test_merge_terms(src, expected):
    d = [x.split('/') for x in src.split(' ')]
    merged = set()
    got = merge_terms(d, skkdict, merged)
    print(merged)
    assert list(got) == expected
