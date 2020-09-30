import pytest

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


@pytest.mark.parametrize('src, expected', [
    ('aka', 'a'),
    ('sona', 'so'),
    ('son', 'so'),
    ('sonn', 'so'),
    ('sonnna', 'sonn'),
    ('sozh', 'so'),
])
def test_remove_last_char(src, expected):
    romkan = RomkanConverter()
    assert romkan.remove_last_char(src) == expected
