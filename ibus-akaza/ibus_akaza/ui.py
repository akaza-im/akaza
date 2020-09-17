import time
from typing import List, Dict

import gi

gi.require_version('IBus', '1.0')

from gi.repository import IBus
from gi.repository import GLib

import sys
import re
import logging
import pathlib

from jaconv import jaconv

from akaza import romkan, Akaza
from akaza.node import Node
from akaza.user_language_model import UserLanguageModel
from akaza_data.system_dict import SystemDict
from akaza_data.system_language_model import SystemLanguageModel
from akaza.user_dict import load_user_dict_from_json_config
from akaza.graph import GraphResolver
from akaza.language_model import LanguageModel

INPUT_MODE_KANA = 1
INPUT_MODE_ALPHA = 2

num_keys = []
for n in range(1, 10):
    num_keys.append(getattr(IBus, str(n)))
num_keys.append(getattr(IBus, '0'))
del n

numpad_keys = []
for n in range(1, 10):
    numpad_keys.append(getattr(IBus, 'KP_' + str(n)))
numpad_keys.append(getattr(IBus, 'KP_0'))
del n


def build_akaza():
    configdir = pathlib.Path(GLib.get_user_config_dir(), 'ibus-akaza')
    user_dict_path = configdir.joinpath('user-dict')
    user_dict_path.mkdir(parents=True, exist_ok=True)

    user_dict_conf_path = configdir.joinpath('user-dict.json')
    logging.info(f"user_dict_conf_path={user_dict_conf_path}")
    if user_dict_conf_path.exists():
        logging.info(f"Loading user dict: {user_dict_conf_path}")
        user_dict = load_user_dict_from_json_config(str(user_dict_conf_path))
    else:
        logging.info(f"Missing user dict: {user_dict_conf_path}")
        user_dict = None

    user_language_model_path = configdir.joinpath('user_language_model')
    user_language_model_path.mkdir(parents=True, exist_ok=True)
    user_language_model = UserLanguageModel(str(user_language_model_path))

    system_dict = SystemDict.load()
    system_language_model = SystemLanguageModel.load()

    language_model = LanguageModel(
        system_language_model=system_language_model,
        user_language_model=user_language_model,
    )

    resolver = GraphResolver(
        system_dict=system_dict,
        user_dict=user_dict,
        language_model=language_model,
    )

    return user_language_model, Akaza(resolver=resolver)


try:
    t0 = time.time()
    user_language_model, akaza = build_akaza()
    logging.info(f"Loaded Akaza in {time.time() - t0} seconds")
except:
    logging.error("Cannot initialize Akaza.", exc_info=True)
    sys.exit(1)


# ----------------------------------------------------------------------
# the engine
# ----------------------------------------------------------------------

class AkazaIBusEngine(IBus.Engine):
    user_language_model: UserLanguageModel
    current_clause: int
    node_selected: Dict[int, int]
    clauses: List[List[Node]]
    prop_list: IBus.PropList
    akaza: Akaza
    mode: int
    force_selected_clause: List[slice]

    __gtype_name__ = 'AkazaIBusEngine'

    def __init__(self):
        super(AkazaIBusEngine, self).__init__()
        self.is_invalidate = False
        # 未確定文字列。
        self.preedit_string = ''
        # 候補文字列
        self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
        self.prop_list = IBus.PropList()
        self.akaza = akaza
        self.user_language_model = user_language_model
        self.logger = logging.getLogger(__name__)
        self.mode = INPUT_MODE_KANA

        # 変換候補。文節ごとの配列。
        self.clauses = []
        # 現在選択されている、文節。
        self.current_clause = 0
        # key は、clause 番号。value は、node の index。
        self.node_selected = {}

        # 文節を選びなおしたもの。
        self.force_selected_clause = []

        # カーソル変更をしたばっかりかどうかを、みるフラグ。
        self.cursor_moved = False

        self.logger.debug("Create Akaza engine OK: 20200916")

        self.input_mode_prop = IBus.Property(key=u'InputMode',
                                             prop_type=IBus.PropType.MENU,
                                             label=IBus.Text.new_from_string("Input mode (あ)"),
                                             icon='',
                                             tooltip=IBus.Text.new_from_string("Switch input mode"),
                                             sensitive=True,
                                             visible=True,
                                             state=IBus.PropState.UNCHECKED,
                                             sub_props=None)
        self.prop_list.append(self.input_mode_prop)

    def set_lookup_table_cursor_pos_in_current_page(self, index):
        """Sets the cursor in the lookup table to index in the current page
g
        Returns True if successful, False if not.
        """
        self.logger.info(f"set_lookup_table_cursor_pos_in_current_page: {index}")
        page_size = self.lookup_table.get_page_size()
        if index > page_size:
            self.logger.info(f"index too big: {index} > {page_size}")
            return False
        page, pos_in_page = divmod(self.lookup_table.get_cursor_pos(),
                                   page_size)
        new_pos = page * page_size + index
        if new_pos > self.lookup_table.get_number_of_candidates():
            self.logger.info(f"new_pos too big: {new_pos} > {self.lookup_table.get_number_of_candidates()}")
            return False
        self.lookup_table.set_cursor_pos(new_pos)
        self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
        return True

    def do_candidate_clicked(self, index, dummy_button, dummy_state):
        if self.set_lookup_table_cursor_pos_in_current_page(index):
            self.commit_candidate()

    def do_process_key_event(self, keyval, keycode, state):
        try:
            return self._do_process_key_event(keyval, keycode, state)
        except:
            self.logger.error(f"Cannot process key event: keyval={keyval} keycode={keycode} state={state}",
                              exc_info=True)
            return False

    def _do_process_key_event(self, keyval, keycode, state):
        # self.logger.debug("process_key_event(%04x, %04x, %04x)" % (keyval, keycode, state))

        # ignore key release events
        is_press = ((state & IBus.ModifierType.RELEASE_MASK) == 0)
        if not is_press:
            return False

        # 入力モードの切り替え機能。
        if keyval == IBus.Henkan:
            self._set_input_mode(INPUT_MODE_KANA)
            return True
        elif keyval == IBus.Muhenkan:
            self._set_input_mode(INPUT_MODE_ALPHA)
            return True

        if self.preedit_string:
            if keyval in (IBus.Return, IBus.KP_Enter):
                if self.in_henkan_mode():
                    self.commit_candidate()
                else:
                    # 無変換状態では、ひらがなに変換してコミットします。
                    self.commit_string(romkan.to_hiragana(self.preedit_string))
                return True
            elif keyval == IBus.Escape:
                self.preedit_string = ''
                self.update_candidates()
                return True
            elif keyval == IBus.BackSpace:
                if self.in_henkan_mode():
                    # 変換中の場合、無変換モードにもどす。
                    self.lookup_table.clear()
                    self.hide_auxiliary_text()
                    self.hide_lookup_table()
                else:
                    # サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
                    self.preedit_string = re.sub('(?:z[hjkl.-]|n+|[kstnhmyrwgzjdbp]?[aiueo]|.)$', '',
                                                 self.preedit_string)
                # 変換していないときのレンダリングをする。
                self.update_preedit_text_before_henkan()
                return True
            elif keyval in num_keys and self.in_henkan_mode():
                # TODO: 変換候補が表示されている状態の時にのみハンドリングされるべき。
                index = num_keys.index(keyval)
                if self.set_lookup_table_cursor_pos_in_current_page(index):
                    self.refresh()
                    return True
                return False
            elif keyval in numpad_keys and self.in_henkan_mode():
                # TODO: 変換候補が表示されている状態の時にのみハンドリングされるべき。
                index = numpad_keys.index(keyval)
                if self.set_lookup_table_cursor_pos_in_current_page(index):
                    self.refresh()
                    return True
                return False
            elif keyval in (IBus.Page_Up, IBus.KP_Page_Up):
                self.page_up()
                return True
            elif keyval in (IBus.Page_Down, IBus.KP_Page_Down):
                self.page_down()
                return True
            elif keyval in (IBus.Up, IBus.KP_Up):
                self.cursor_up()
                return True
            elif keyval in (IBus.Down, IBus.KP_Down):
                self.cursor_down()
                return True
            elif keyval in (IBus.Right, IBus.KP_Right):
                if state & IBus.ModifierType.SHIFT_MASK == 0:
                    self.cursor_right()
                else:
                    self.extend_clause_right()
                return True
            elif keyval in (IBus.Left, IBus.KP_Left):
                if state & IBus.ModifierType.SHIFT_MASK == 0:
                    self.cursor_left()
                else:
                    self.extend_clause_left()
                return True
            elif keyval == IBus.F6:
                # F6 convert selected word/characters to full-width hiragana (standard hiragana): ホワイト → ほわいと
                self.convert_to_full_hiragana()
                return True
            elif keyval == IBus.F7:
                # F7 convert to full-width katakana (standard katakana): ほわいと → ホワイト
                self.convert_to_full_katakana()
                return True
            elif keyval == IBus.F8:
                # F8 convert to half-width katakana (katakana for specific purpose): ホワイト → ﾎﾜｲﾄ
                self.convert_to_half_katakana()
                return True
            elif keyval == IBus.F9:
                # F9 convert to full-width romaji, all-capitals, proper noun capitalization (latin script inside
                # Japanese text): ホワイト → ｈｏｗａｉｔｏ → ＨＯＷＡＩＴＯ → Ｈｏｗａｉｔｏ
                self.convert_to_full_romaji()
                return True
            elif keyval == IBus.F10:
                # F10 convert to half-width romaji, all-capitals, proper noun capitalization (latin script like
                # standard English): ホワイト → howaito → HOWAITO → Howaito
                self.convert_to_half_romaji()
                return True

        # スペース
        if keyval == IBus.space:
            if len(self.preedit_string) == 0:
                # もし、まだなにもはいっていなければ、ただの空白をそのままいれる。
                return False
            else:
                if self.in_henkan_mode():
                    self.logger.debug("cursor down.")
                    self.cursor_down()
                else:
                    # 実際に変換していく。
                    self.logger.debug("update_candidates.")
                    self.update_candidates()
                return True

        if self.mode == INPUT_MODE_KANA:
            # Allow typing all ASCII letters and punctuation
            if ord('!') <= keyval <= ord('~'):
                if state & (IBus.ModifierType.CONTROL_MASK | IBus.ModifierType.MOD1_MASK) == 0:
                    if self.in_henkan_mode():
                        self.commit_candidate()

                    self.preedit_string += chr(keyval)
                    # この時点では、preedit string にだけ、追加して表示するひつようがあります。
                    self.update_preedit_text_before_henkan()
                    return True
            else:
                if keyval < 128 and self.preedit_string:
                    self.commit_string(self.preedit_string)
        else:
            return False

        return False

    def _set_input_mode(self, mode):
        if mode == INPUT_MODE_KANA:
            self.logger.info("Switch to kana mode")
            self.input_mode_prop.set_label(IBus.Text.new_from_string("Input mode (あ)"))
        elif mode == INPUT_MODE_ALPHA:
            self.logger.info("Switch to alpha mode")
            self.input_mode_prop.set_label(IBus.Text.new_from_string("Input mode (A)"))
        else:
            raise RuntimeError(f"Unknown input mode: {mode}")
        self.register_properties(self.prop_list)
        self.mode = INPUT_MODE_ALPHA


def in_henkan_mode(self):
    return self.lookup_table.get_number_of_candidates() > 0


def convert_to_full_katakana(self):
    self.logger.info("Convert to full katakana")

    # カタカナ候補のみを表示するようにする。
    hira = romkan.to_hiragana(self.preedit_string)
    kata = jaconv.hira2kata(hira)

    self.convert_to_single(hira, kata)


def convert_to_full_hiragana(self):
    self.logger.info("Convert to full hiragana")

    # カタカナ候補のみを表示するようにする。
    hira = romkan.to_hiragana(self.preedit_string)
    self.convert_to_single(hira, hira)


def convert_to_half_katakana(self):
    self.logger.info("Convert to half katakana")

    # 半角カタカナ候補のみを表示するようにする。
    hira = romkan.to_hiragana(self.preedit_string)
    kata = jaconv.hira2kata(hira)
    kata = jaconv.z2h(kata)

    self.convert_to_single(hira, kata)


def convert_to_half_romaji(self):
    self.logger.info("Convert to half romaji")

    # 半角カタカナ候補のみを表示するようにする。
    hira = romkan.to_hiragana(self.preedit_string)
    romaji = jaconv.z2h(self.preedit_string)

    self.convert_to_single(hira, romaji)


def convert_to_full_romaji(self):
    self.logger.info("Convert to full romaji")

    hira = romkan.to_hiragana(self.preedit_string)
    romaji = jaconv.h2z(self.preedit_string, kana=True, digit=True, ascii=True)

    self.convert_to_single(hira, romaji)


def convert_to_single(self, yomi, word) -> None:
    """
    特定の1文節の文章を候補として表示する。
    F6 などを押した時用。
    """
    # 候補を設定
    self.clauses = [[Node(start_pos=0, word=word, yomi=yomi)]]
    self.current_clause = 0
    self.node_selected = {}
    self.force_selected_clause = []

    # ルックアップテーブルに候補を設定
    self.lookup_table.clear()
    candidate = IBus.Text.new_from_string(word)
    self.lookup_table.append_candidate(candidate)

    # 表示を更新
    self.refresh()


def invalidate(self):
    if self.is_invalidate:
        return
    self.is_invalidate = True
    GLib.idle_add(self.update_candidates)


def page_up(self):
    if self.lookup_table.page_up():
        self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
        self.cursor_moved = True
        self.refresh()
        return True
    return False


def page_down(self):
    if self.lookup_table.page_down():
        self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
        self.cursor_moved = True
        self.refresh()
        return True
    return False


def cursor_up(self):
    if self.lookup_table.cursor_up():
        self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
        self.cursor_moved = True
        self.refresh()
        return True
    return False


def cursor_down(self):
    """
    次の変換候補を選択する。
    """
    if self.lookup_table.cursor_down():
        self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
        self.cursor_moved = True
        self.refresh()
        return True
    return False

    # 選択する分節を右にずらす。


def cursor_right(self):
    """
    選択する分節を右にずらす。
    """
    self.logger.info(f"right cursor")

    # いっこしか分節がない場合は、何もせぬ
    if len(self.clauses) == 0:
        self.logger.info(f"right cursor：no clauses")
        return False

    # 既に一番右だった場合、一番左にいく。
    if self.current_clause == len(self.clauses) - 1:
        self.current_clause = 0
    else:
        self.current_clause += 1

    self.cursor_moved = True
    self.create_lookup_table()

    self.refresh()

    # 選択する分節を左にずらす。


def cursor_left(self):
    self.logger.info(f"left cursor")
    # いっこしか分節がない場合は、何もせぬ
    if len(self.clauses) == 0:
        self.logger.info(f"left cursor：no clauses")
        return False

    # 既に一番左だった場合、一番右にいく。
    if self.current_clause == 0:
        self.current_clause = len(self.clauses) - 1
    else:
        self.current_clause -= 1

    self.cursor_moved = True
    self.create_lookup_table()

    self.refresh()


def extend_clause_right(self):
    """
    文節の選択範囲を広げることを支持する
    """
    if len(self.clauses) == 0:
        return False

    max_len = max([clause[0].start_pos + len(clause[0].yomi) for clause in self.clauses])

    self.force_selected_clause = []
    for i, clause in enumerate(self.clauses):
        node = clause[0]
        if self.current_clause == i:
            # 現在選択中の文節の場合、伸ばす。
            self.force_selected_clause.append(
                slice(node.start_pos, min(node.start_pos + len(node.yomi) + 1, max_len)))
        elif self.current_clause + 1 == i:
            # 次の分節を一文字ヘラス
            self.force_selected_clause.append(
                slice(node.start_pos + 1, node.start_pos + len(node.yomi)))
        else:
            # それ以外は現在指定の分節のまま
            self.force_selected_clause.append(
                slice(node.start_pos, node.start_pos + len(node.yomi)))

    self.force_selected_clause = [x for x in self.force_selected_clause if x.start != x.stop]
    self._update_candidates()
    # TODO: もし、分節の長さをいじったら、self.node_selected も変更するべき。


def extend_clause_left(self):
    """
    文節の選択範囲を広げることを支持する
    """
    if len(self.clauses) == 0:
        return False

    # 一番左の文節にフォーカスがある場合、一番左の文節が短くなるべき。
    target_clause = 1 if self.current_clause == 0 and len(self.clauses) > 1 else self.current_clause

    self.force_selected_clause = []
    for i, clause in enumerate(self.clauses):
        node = clause[0]
        if target_clause == i:
            # 現在選択中の文節の場合、伸ばす。
            self.force_selected_clause.append(
                slice(node.start_pos - 1, node.start_pos + len(node.yomi)))
        elif target_clause - 1 == i:
            # 前の分節を一文字ヘラス
            self.force_selected_clause.append(
                slice(node.start_pos, node.start_pos + len(node.yomi) - 1))
        else:
            # それ以外は現在指定の分節のまま
            self.force_selected_clause.append(
                slice(node.start_pos, node.start_pos + len(node.yomi)))

    self.force_selected_clause = [x for x in self.force_selected_clause if x.start != x.stop]
    self._update_candidates()
    # TODO: もし、分節の長さをいじったら、self.node_selected も変更するべき。


def commit_string(self, text):
    self.logger.info("commit_string.")
    self.cursor_moved = False

    if self.in_henkan_mode():
        # 変換モードのときのみ学習を実施する。
        candidate_nodes = []
        for clauseid, nodes in enumerate(self.clauses):
            candidate_nodes.append(nodes[self.node_selected.get(clauseid, 0)])
        self.user_language_model.add_entry(candidate_nodes)
        # user language model の書き出しは、バックグラウンドスレッドでやりたい。
        self.logger.info("Save user language model.")
        self.user_language_model.save()

    self.commit_text(IBus.Text.new_from_string(text))

    self.preedit_string = ''
    self.clauses = []
    self.current_clause = 0
    self.node_selected = {}
    self.force_selected_clause = []

    self.update_candidates()


def build_string(self):
    result = ''
    for clauseid, nodes in enumerate(self.clauses):
        result += nodes[self.node_selected.get(clauseid, 0)].word
    return result


def commit_candidate(self):
    self.logger.info("commit_candidate")
    s = self.build_string()
    self.logger.info(f"Committing {s}")
    self.commit_string(s)


def update_candidates(self):
    self.logger.info("update_candidates")
    try:
        self._update_candidates()
        self.current_clause = 0
        self.node_selected = {}
    except:
        self.logger.error(f"cannot get kanji candidates {sys.exc_info()[0]}", exc_info=True)


def _update_candidates(self):
    if len(self.preedit_string) > 0:
        # 変換をかける
        self.clauses = self.akaza.convert(self.preedit_string, self.force_selected_clause)
    else:
        self.clauses = []
    self.create_lookup_table()

    self.refresh()


def refresh(self):
    preedit_len = len(self.preedit_string)

    if len(self.clauses) == 0:
        self.hide_auxiliary_text()
        self.hide_lookup_table()
        self.hide_preedit_text()
        return

    current_clause = self.clauses[self.current_clause]
    current_node = current_clause[0]

    # -- auxiliary text(ポップアップしてるやつのほう)
    first_candidate = current_node.yomi
    auxiliary_text = IBus.Text.new_from_string(first_candidate)
    auxiliary_text.set_attributes(IBus.AttrList())
    self.update_auxiliary_text(auxiliary_text, preedit_len > 0)

    text = self.build_string()
    preedit_attrs = IBus.AttrList()
    # 全部に下線をひく。
    preedit_attrs.append(IBus.Attribute.new(IBus.AttrType.UNDERLINE,
                                            IBus.AttrUnderline.SINGLE, 0, len(text)))
    bgstart = sum([len(self.clauses[i][0].word) for i in range(0, self.current_clause)])
    # 背景色を設定する。
    preedit_attrs.append(IBus.Attribute.new(
        IBus.AttrType.BACKGROUND,
        0x00333333,
        bgstart,
        bgstart + len(current_node.word)))
    preedit_text = IBus.Text.new_from_string(text)
    preedit_text.set_attributes(preedit_attrs)
    self.update_preedit_text(preedit_text, len(text), len(text) > 0)

    # 候補があれば、選択肢を表示させる。
    self._update_lookup_table()
    self.is_invalidate = False


def update_preedit_text_before_henkan(self):
    """
    無変換状態で、どんどん入力していくフェーズ。
    """

    if len(self.preedit_string) == 0:
        self.hide_preedit_text()
        return

    # 平仮名にする。
    text = romkan.to_hiragana(self.preedit_string)
    self.clauses = [
        [Node(word=text, yomi=text, start_pos=3)]
    ]
    self.current_clause = 0

    preedit_attrs = IBus.AttrList()
    preedit_attrs.append(IBus.Attribute.new(IBus.AttrType.UNDERLINE,
                                            IBus.AttrUnderline.SINGLE, 0, len(text)))
    preedit_text = IBus.Text.new_from_string(text)
    preedit_text.set_attributes(preedit_attrs)
    self.update_preedit_text(preedit_text, len(text), len(text) > 0)


def create_lookup_table(self):
    """
    現在の候補選択状態から、 lookup table を構築する。
    """
    # 一旦、ルックアップテーブルをクリアする
    self.lookup_table.clear()

    # 現在の未変換情報を元に、候補を産出していく。
    if len(self.clauses) > 0:
        # lookup table に候補を詰め込んでいく。
        for node in self.clauses[self.current_clause]:
            candidate = IBus.Text.new_from_string(node.word)
            self.lookup_table.append_candidate(candidate)


def _update_lookup_table(self):
    """
    候補があれば lookup table を表示。なければ非表示にする。
    """
    visible = self.lookup_table.get_number_of_candidates() > 0
    self.update_lookup_table(self.lookup_table, visible)


def do_focus_in(self):
    # self.logger.debug("focus_in")
    self.register_properties(self.prop_list)


def do_focus_out(self):
    # self.logger.debug("focus_out")
    self.do_reset()


def do_reset(self):
    # self.logger.debug("reset")
    self.preedit_string = ''
    self.force_selected_clause = []
    self.clauses = []
    self.current_clause = 0
    self.lookup_table.clear()
    self.hide_auxiliary_text()
    self.hide_lookup_table()


def do_property_activate(self, prop_name):
    self.logger.debug("PropertyActivate(%s)" % prop_name)


def do_page_up(self):
    return self.page_up()


def do_page_down(self):
    return self.page_down()


def do_cursor_up(self):
    return self.cursor_up()


def do_cursor_down(self):
    return self.cursor_down()
