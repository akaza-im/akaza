from akaza_data.emoji import EmojiDict

emoji_dict = EmojiDict.load()


def test_system_dict():
    s = emoji_dict['すし']
    assert '🍣' in s
