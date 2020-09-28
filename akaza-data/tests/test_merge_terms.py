import sys
import pathlib
import pytest

path = str(pathlib.Path(__file__).parent.parent.joinpath('bin'))
sys.path.append(path)

from akaza_data_utils.merge_terms import merge_terms, load_skk_dict

skkdict = load_skk_dict()


@pytest.mark.parametrize('src, expected', [
    ('小/接頭辞/しょう 学校/名詞/がっこう', [('小学校', 'しょうがっこう')]),
    ('書/動詞/か く/語尾/く', [('書く', 'かく')]),
    ('書/動詞/か い/語尾/い て/助詞/て い/動詞/い た/助動詞/た もの/名詞/もの で/助動詞/で あ/動詞/あ る/語尾/る', [
        ('書いて', 'かいて'),
        ('いた', 'いた'),
        ('もの', 'もの'),
        ('で', 'で'),
        ('ある', 'ある'),
    ])
])
def test_merge_terms(src, expected):
    d = [x.split('/') for x in src.split(' ')]
    merged = set()
    got = list(merge_terms(d, skkdict, merged))
    print(merged)
    print(got)
    assert got == expected
