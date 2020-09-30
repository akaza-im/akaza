from akaza_data.system_dict import SystemDict

system_dict = SystemDict.load()


def test_system_dict():
    s = system_dict['れいわ']
    assert '令和' in s

    beer = system_dict['びーる']
    assert '🍻' not in beer


def test_system_dict2():
    system_dict = SystemDict.load()
    assert system_dict.prefixes('あいう') == ['あ', 'あい', 'あいう']
    assert system_dict['あいう'] == ['藍宇']
    assert len(system_dict['あい']) > 7


def test_has_item():
    system_dict = SystemDict.load()
    assert system_dict.has_item('あいう')
    assert not system_dict.has_item('あいうじゃぱぱぱーん')


def test_prefixes():
    system_dict = SystemDict.load()
    assert system_dict.prefixes('あい') == ['あ', 'あい']
