import pytest
import sys

sys.path.append('../akaza-data/')

from akaza.romkan import RomkanConverter


def test_foo():
    romkan = RomkanConverter()
    assert romkan.to_hiragana('a') == 'あ'
    assert romkan.to_hiragana('ba') == 'ば'
    assert romkan.to_hiragana('hi') == 'ひ'
    assert romkan.to_hiragana('wahaha') == 'わはは'
    assert romkan.to_hiragana('thi') == 'てぃ'
    assert romkan.to_hiragana('better') == 'べってr'
    assert romkan.to_hiragana('[') == '「'
    assert romkan.to_hiragana(']') == '」'


@pytest.mark.parametrize('src, expected', [
    ('a', 'あ'),
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

# ROMKAN_H.update({})
