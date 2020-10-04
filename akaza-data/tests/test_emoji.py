from akaza_data.systemlm_loader import BinaryDict

emoji_dict = BinaryDict()
emoji_dict.load("akaza_data/data/single_term.trie")


def test_system_dict():
    s = emoji_dict.find_kanjis('すし')
    assert '🍣' in s
