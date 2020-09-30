from typing import Dict

from gi.repository import GLib

import yaml
import logging
import pathlib

from akaza.skk_file_dict import load_skk_file_dict


def load_config() -> Dict:
    configdir = pathlib.Path(GLib.get_user_config_dir(), 'ibus-akaza')
    configfile = configdir.joinpath('config.yml')
    if configfile.exists():
        with configfile.open('r') as rfp:
            src = rfp.read()
            return yaml.load(src, Loader=yaml.Loader)
    return {}


class ConfigLoader:
    def __init__(self, logger=logging.getLogger(__name__)):
        self._data = load_config()
        self.logger = logger

    def load_user_dict(self):
        for user_dict in self._data.get('user_dicts', []):
            path = user_dict['path']
            encoding = user_dict.get('encoding', 'utf-8')
            self.logger.info(f"Loading path={path}, encoding={encoding}.")
            yield load_skk_file_dict(path, encoding)

    def get(self, key, default=None):
        if default is not None:
            return self._data.get(key, default)
        else:
            return self._data.get(key)
