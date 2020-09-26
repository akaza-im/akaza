from akaza_data import SystemLanguageModel


def test_unigram():
    system_language_model = SystemLanguageModel.load()
    assert system_language_model.get_unigram_cost('愛/あい') != system_language_model.get_unigram_cost('安威/あい')


def test_unigram_siin():
    system_language_model = SystemLanguageModel.load()
    assert system_language_model.get_unigram_cost('子音/しいん') != system_language_model.get_unigram_cost('試飲/しいん')
