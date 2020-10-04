import pathlib

from akaza_data.systemlm_loader import BinaryDict


class SystemDict:
    _trie: BinaryDict

    def __init__(self, trie: BinaryDict):
        assert trie is not str
        self._trie = trie

    @staticmethod
    def load(path: str = str(pathlib.Path(__file__).parent.absolute().joinpath('data/system_dict.trie'))):
        print(path)
        trie = BinaryDict()
        trie.load(path)
        return SystemDict(trie)

    def prefixes(self, yomi):
        return self._trie.prefixes(yomi)

    def __getitem__(self, yomi):
        return self._trie.find_kanjis(yomi)

    def has_item(self, yomi):
        return len(self._trie.find_kanjis(yomi)) > 0
