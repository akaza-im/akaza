from comb.system_dict import SystemDict

system_dict = SystemDict('model/system_dict.trie')


def test_system_dict():
    s = system_dict['れいわ']
    assert '令和' in s

    # 絵文字辞書
    s = system_dict['びーる']
    assert '🍺' in s
