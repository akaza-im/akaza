import pytest
import sys

sys.path.append('../akaza-data/')

from akaza.romkan import to_hiragana


def test_foo():
    assert to_hiragana('a') == 'あ'
    assert to_hiragana('ba') == 'ば'
    assert to_hiragana('hi') == 'ひ'
    assert to_hiragana('wahaha') == 'わはは'
    assert to_hiragana('thi') == 'てぃ'
    assert to_hiragana('better') == 'べってr'
    assert to_hiragana('[') == '「'
    assert to_hiragana(']') == '」'


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
])
def test_bar(src, expected):
    assert to_hiragana(src) == expected

