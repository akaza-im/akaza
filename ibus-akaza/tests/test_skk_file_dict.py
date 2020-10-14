import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../pyakaza/').absolute().resolve()))

from ibus_akaza.skk_file_dict import load_skk_file_dict


def test_read2():
    path = str(pathlib.Path(__file__).parent.joinpath('data', 'SKK-JISYO.test'))
    d = load_skk_file_dict(path)
    assert d.find_kanjis('たばた') == ['田端']
