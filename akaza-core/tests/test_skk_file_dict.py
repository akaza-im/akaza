from tempfile import TemporaryDirectory
import pathlib
import sys

sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.skk_file_dict import load_skk_file_dict


def test_read2():
    tmpdir = TemporaryDirectory()

    path = str(pathlib.Path(__file__).parent.joinpath('data', 'SKK-JISYO.test'))
    d = load_skk_file_dict(path)
    assert d['たばた'] == ['田端']
    assert d.has_item('たばた')
    assert d.prefixes('たばた') == ['た', 'たば', 'たばた']
