from akaza_data import SystemDict, SystemLanguageModel


def test_system_dict():
    system_dict = SystemDict.load()
    assert system_dict.prefixes('あいう') == ['あ', 'あい', 'あいう']
    assert system_dict['あいう'] == ['藍宇']
    assert len(system_dict['あい']) > 7


def test_system_language_model():
    system_language_model = SystemLanguageModel.load()
    assert system_language_model.get_unigram_cost('愛') == -11.0
