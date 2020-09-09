from comb.skkdict import merge_skkdict

def test_merge_skkdict():
    got = merge_skkdict([
        {'は': ['派', '葉']},
        {'は': ['波', '葉'], 'な': ['菜']},
    ])
    print(got)
    assert got == {'は': ['派', '葉', '波'], 'な': ['菜']}

