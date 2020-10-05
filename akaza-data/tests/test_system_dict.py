from akaza_data.systemlm_loader import BinaryDict

system_dict = BinaryDict()
system_dict.load("akaza_data/data/system_dict.trie")


def test_system_dict():
    s = system_dict.find_kanjis('れいわ')
    assert '令和' in s

    beer = system_dict.find_kanjis('びーる')
    assert '🍻' not in beer


def test_system_dict2():
    assert system_dict.prefixes('あいう') == ['あ', 'あい', 'あいう']
    assert system_dict.find_kanjis('あいう') == ['藍宇']
    assert len(system_dict.find_kanjis('あい')) > 7


def test_prefixes():
    assert system_dict.prefixes('あい') == ['あ', 'あい']
