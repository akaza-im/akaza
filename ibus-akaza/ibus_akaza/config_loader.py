from typing import Dict

from gi.repository import GLib

import yaml
import pathlib


def load_config() -> Dict:
    configdir = pathlib.Path(GLib.get_user_config_dir(), 'ibus-akaza')
    configfile = configdir.joinpath('config.yml')
    if configfile.exists():
        with configfile.open('r') as rfp:
            src = rfp.read()
            return yaml.load(src)
    return {}
