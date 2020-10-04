from typing import List

from akaza_data.systemlm_loader import BinaryDict


# XXX Is there any reason to implement this?
class Dictionary:
    # 通常辞書リスト
    # 単文節辞書リスト
    def __init__(
            self,
            normal_dicts: List[BinaryDict]):
        self.normal_dicts = normal_dicts

    def prefixes(self, yomi: str):
        prefixes = []
        for normal_dict in self.normal_dicts:
            prefixes += normal_dict.prefixes(yomi)
        return set(prefixes)

    def __getitem__(self, yomi):
        result = []
        for normal_dict in self.normal_dicts:
            for kanji in normal_dict.find_kanjis(yomi):
                if kanji not in result:
                    result.append(kanji)
        return result

    def has_item(self, yomi):
        for normal_dict in self.normal_dicts:
            if normal_dict.has_item(yomi):
                return True
