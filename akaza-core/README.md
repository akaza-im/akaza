# Akaza

## What's this?

Yet another kana-kanji conversion system written in Python 3.

## How do I use it?

### Use as a library

    system_language_model = SystemLanguageModel.load()
    system_dict = SystemDictionary.load()
    akaza = Akaza(
        system_language_model = system_language_model,
        system_dict: system_dict,
        user_language_model: user_language_model,
        user_dict: None,
    )
    print(akaza.convert('watasinonamaehanakanodesu.'))
    # → 私の名前は中野です。
