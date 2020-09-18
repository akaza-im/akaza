import os
import sys
import pathlib
import pytest

sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))
sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-core/').absolute().resolve()))

os.environ['AKAZA_DICTIONARY_DIR'] = 'model/'
os.environ['AKAZA_MODEL_DIR'] = 'model/'

from akaza.node import Node
from ibus_akaza.ui import AkazaIBusEngine
from ibus_akaza.input_mode import INPUT_MODE_KATAKANA, INPUT_MODE_HIRAGANA


def test_extend_clause_right():
    ui = AkazaIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    # AS-IS:
    #   タノシ/イ/ジカン
    #   ↑ focus
    # TO-BE:
    #   タノシ/イ/ジカン
    #         ↑ focus
    ui.cursor_right()

    # 「イジ」に伸びるはず
    # タノシ/イジ/カン
    # 0 1 2 3 4 5 6
    # slice(0,3)
    # slice(3,5)
    # slice(5,7)
    ui.extend_clause_right()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノシ/イジ/カン'

    # 2文節目が イジ になっている
    assert '維持' in [node.word for node in ui.clauses[1]]


def test_extend_clause_right_most_right():
    ui = AkazaIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    # AS-IS:
    #   タノシ/イ/ジカン
    #   ↑ focus
    # TO-BE:
    #   タノシ/イ/ジカン
    #           ↑ focus
    ui.cursor_right()
    ui.cursor_right()

    # すでに一番右なので何も行われない
    # タノシ/イ/ジカン
    ui.extend_clause_right()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノシ/イ/ジカン'


def test_extend_clause_left():
    ui = AkazaIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    # AS-IS:
    #   タノシ/イ/ジカン
    #   ↑ focus
    # TO-BE:
    #   タノシ/イ/ジカン
    #         ↑ focus
    ui.cursor_right()

    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # タノ/シイ/ジカン
    # 0 1 2 3 4 5 6
    # slice(0,2)
    # slice(2,4)
    # slice(4,7)
    ui.extend_clause_left()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノ/シイ/ジカン'

    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # 2文節目が しい になっている
    assert 'たの/しい/じかん' == '/'.join([clause[0].yomi for clause in ui.clauses])


def test_extend_clause_left_most_left():
    ui = AkazaIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    #   タノシ/イ/ジカン
    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # タノ/シイ/ジカン
    # 0 1 2 3 4 5 6
    # slice(0,2)
    # slice(2,4)
    # slice(4,7)
    ui.extend_clause_left()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノ/シイ/ジカン'

    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # 2文節目が しい になっている
    assert 'たの/しい/じかん' == '/'.join([clause[0].yomi for clause in ui.clauses])


@pytest.mark.skip(reason='今はうごかない')
def test_extend_clause_left_most_left_and_more():
    ui = AkazaIBusEngine()
    ui.preedit_string = "dondaketochikan"  # どん/だけ/とち/かん
    ui.update_candidates()

    # どん/だけ/とち/かん
    assert '/'.join([clause[0].yomi for clause in ui.clauses]) == 'どん/だけ/と/ちかん'

    # どん/だけ/とち/かん
    # 0 1 2 3 4 5 6
    # slice(0,2)
    # slice(2,4)
    # slice(4,7)
    assert ui.current_clause == 0
    ui.cursor_right()  # focus to だけ
    assert ui.current_clause == 1
    ui.cursor_right()  # focus to とち
    assert ui.current_clause == 2
    ui.extend_clause_right()  # とち→とちか
    assert '/'.join([clause[0].yomi for clause in ui.clauses]) == 'どん/だけ/とちか/ん'
    assert '/'.join(['どんだけとちかん'[s] for s in ui.force_selected_clause]) == 'どん/だけ/とちか/ん'
    assert ui.current_clause == 2
    ui.extend_clause_right()  # とちか→とちかん
    assert '/'.join([clause[0].yomi for clause in ui.clauses]) == 'どん/だけ/とちかん'


def test_update_preedit_text_before_henkan1():
    ui = AkazaIBusEngine()
    ui._set_input_mode(INPUT_MODE_HIRAGANA)
    ui.preedit_string = "hyoi"
    ui.update_preedit_text_before_henkan()
    print(ui.clauses)
    assert [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ] == [
               [Node(word='ひょい', yomi='ひょい', start_pos=0)]
           ]
    assert ui.clauses == [
        [Node(word='ひょい', yomi='ひょい', start_pos=0)]
    ]


def test_update_preedit_text_before_henkan2():
    ui = AkazaIBusEngine()
    ui._set_input_mode(INPUT_MODE_KATAKANA)
    ui.preedit_string = "hyoi-"
    ui.update_preedit_text_before_henkan()
    assert ui.clauses == [
        [Node(word='ヒョイー', yomi='ひょいー', start_pos=0)]
    ]
