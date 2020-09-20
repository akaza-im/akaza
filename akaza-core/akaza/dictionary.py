from typing import List

from akaza.skk_file_dict import SkkFileDict
from akaza_data.system_dict import SystemDict


class Dictionary:
    def __init__(
            self,
            system_dict: SystemDict,
            user_dicts: List[SkkFileDict]):
        self.system_dict = system_dict

        if user_dicts is None:
            self.user_dicts = []
        else:
            self.user_dicts = user_dicts

    def prefixes(self, yomi: str):
        prefixes = self.system_dict.prefixes(yomi)
        for user_dict in self.user_dicts:
            prefixes += user_dict.prefixes(yomi)
        return set(prefixes)

    def __getitem__(self, yomi):
        # print(f"SkkFileDict: {yomi}---- __getitem__")
        result = []
        for user_dict in self.user_dicts:
            if user_dict.has_item(yomi):
                for word in user_dict[yomi]:
                    if word not in result:
                        result.append(word)
        if self.system_dict.has_item(yomi):
            for word in self.system_dict[yomi]:
                if word not in result:
                    result.append(word)
        return result

    def has_item(self, yomi):
        if self.system_dict.has_item(yomi):
            return True
        for user_dict in self.user_dicts:
            if user_dict.has_item(yomi):
                return True
