import pytest
import pathlib
import sys

sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.romkan import RomkanConverter


@pytest.mark.parametrize('src, expected', [
    ('a', 'あ'),
    ('ba', 'ば'),
    ('hi', 'ひ'),
    ('wahaha', 'わはは'),
    ('thi', 'てぃ'),
    ('better', 'べってr'),
    ('[', '「'),
    (']', '」'),
    ('wo', 'を'),
    ('du', 'づ'),
    ('we', 'うぇ'),
    ('di', 'ぢ'),
    ('fu', 'ふ'),
    ('ti', 'ち'),
    ('wi', 'うぃ'),
    ('we', 'うぇ'),
    ('wo', 'を'),
    ('z,', '‥'),
    ('z.', '…'),
    ('z/', '・'),
    ('z[', '『'),
    ('z]', '』'),
    ('du', 'づ'),
    ("di", "ぢ"),
    ("fu", "ふ"),
    ("ti", "ち"),
    ("wi", "うぃ"),
    ("we", "うぇ"),
    ("wo", "を"),
])
def test_bar(src, expected):
    romkan = RomkanConverter()
    assert romkan.to_hiragana(src) == expected
