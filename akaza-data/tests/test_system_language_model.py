from akaza_data import SystemDict, SystemLanguageModel


def test_unigram():
    system_language_model = SystemLanguageModel.load()
    assert system_language_model.get_unigram_cost('愛/あい') != system_language_model.get_unigram_cost('安威/あい')
