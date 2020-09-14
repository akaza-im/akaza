from skkdictutils import merge_skkdict, expand_okuri


def test_merge_skkdict():
    got = merge_skkdict([
        {'は': ['派', '葉']},
        {'は': ['波', '葉'], 'な': ['菜']},
    ])
    print(got)
    assert got == {'は': ['派', '葉', '波'], 'な': ['菜']}


def test_expand_okuri():
    got = list(expand_okuri('あいしあw', ['愛し合']))
    print(got)
    assert got == [
        ('あいしあわ', ['愛し合わ']),
        ('あいしあうぃ', ['愛し合うぃ']),
        ('あいしあうぇ', ['愛し合うぇ']),
        ('あいしあを', ['愛し合を']),
    ]
