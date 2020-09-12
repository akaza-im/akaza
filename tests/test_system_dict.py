from comb.system_dict import SystemDict

system_dict = SystemDict()


def test_system_dict():
    d = list(system_dict.get_candidates('better', 'べってr'))
    assert d[0] == 'ベター'

    d = list(system_dict.get_candidates('reiwa', 'れいわ'))
    assert '令和' in d
