import logging
import re
import time
from logging import Logger
from typing import List

import jaconv
from akaza.graph_resolver import GraphResolver
from akaza_data.systemlm_loader import Node, UserLanguageModel

from akaza.romkan import RomkanConverter



class Akaza:
    resolver: GraphResolver
    logger: Logger

    def __init__(self,
                 resolver: GraphResolver,
                 romkan: RomkanConverter,
                 logger: Logger = logging.getLogger(__name__)):
        self.logger = logger
        self.resolver = resolver
        self.romkan = romkan

    # 連文節変換するバージョン。
    def convert(self, src: str, force_selected_clause: List[slice] = None) -> List[List[Node]]:
        self.logger.info(f"convert: {force_selected_clause}")

