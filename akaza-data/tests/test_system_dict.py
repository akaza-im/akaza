from akaza_data.system_dict import SystemDict

system_dict = SystemDict.load()


def test_system_dict():
    s = system_dict['れいわ']
    assert '令和' in s

    # 絵文字辞書
    s = system_dict['びーる']
    assert '🍺' in s


def test_system_dict2():
    system_dict = SystemDict.load()
    assert system_dict.prefixes('あいう') == ['あ', 'あい', 'あいう']
    assert system_dict['あいう'] == ['藍宇']
    assert len(system_dict['あい']) > 7
