from combromkan import to_hiragana


def test_foo():
    assert to_hiragana('a') == 'あ'
    assert to_hiragana('ba') == 'ば'
    assert to_hiragana('hi') == 'ひ'
    assert to_hiragana('wahaha') == 'わはは'
    assert to_hiragana('thi') == 'てぃ'
