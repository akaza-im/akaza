from typing import List, Any, Dict

from gi.repository import IBus
from gi.repository import GLib

import os
import sys
import re
import logging
import pathlib

from comb.engine import Comb
from comb.graph import Node
from comb.user_dict import UserDict
from comb.system_dict import SystemDict

MODE_KANA = 1
MODE_ALPHA = 2

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

configdir = os.path.join(GLib.get_user_config_dir(), 'ibus-comb')
pathlib.Path(configdir).mkdir(parents=True, exist_ok=True)
logging.info(f"Loading user dictionary: {configdir}")
user_dict = UserDict(os.path.join(configdir, 'user-dict.txt'), logging.getLogger('UserDict'))
logging.info("Loaded user dictionary")

system_dict = SystemDict()
logging.info("Loaded system dictionary")

try:
    comb = Comb(logging.getLogger('Comb'), user_dict, system_dict)
    logging.info("Finished Comb.")
except:
    logging.error("Cannot initialize.", exc_info=True)
    sys.exit(1)


# ----------------------------------------------------------------------
# the engine
# ----------------------------------------------------------------------

class CombIBusEngine(IBus.Engine):
    current_clause: int
    node_selected: Dict[int, int]
    current_clause: int
    clauses: List[List[Node]]
    prop_list: IBus.PropList
    comb: Comb
    mode: bool

    __gtype_name__ = 'CombIBusEngine'

    def __init__(self):
        super(CombIBusEngine, self).__init__()
        self.is_invalidate = False
        # 未確定文字列。
        self.preedit_string = ''
        # 候補文字列
        self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
        self.prop_list = IBus.PropList()
        self.comb = comb
        self.user_dict = user_dict
        self.logger = logging.getLogger(__name__)
        self.mode = MODE_KANA
        # 変換候補。文節ごとの配列。
        self.clauses = []
        # 現在選択されている、文節。
        self.current_clause = 0
        # key は、clause 番号。value は、node の index。
        self.node_selected = {}

        # カーソル変更をしたばっかりかどうかを、みるフラグ。
        self.cursor_moved = False

        self.logger.debug("Create Comb engine OK")

    def set_lookup_table_cursor_pos_in_current_page(self, index):
        """Sets the cursor in the lookup table to index in the current page

        Returns True if successful, False if not.
        """
        page_size = self.lookup_table.get_page_size()
        if index > page_size:
            return False
        page, pos_in_page = divmod(self.lookup_table.get_cursor_pos(),
                                   page_size)
        new_pos = page * page_size + index
        if new_pos > self.lookup_table.get_number_of_candidates():
            return False
        self.lookup_table.set_cursor_pos(new_pos)
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
        self.logger.debug("process_key_event(%04x, %04x, %04x)" % (keyval, keycode, state))

        # ignore key release events
        is_press = ((state & IBus.ModifierType.RELEASE_MASK) == 0)
        if not is_press:
            return False

        # 入力モードの切り替え機能。
        if keyval == IBus.Henkan:
            self.logger.info("Switch to kana mode")
            self.mode = MODE_KANA
            return True
        elif keyval == IBus.Muhenkan:
            self.logger.info("Switch to alpha mode")
            self.mode = MODE_ALPHA
            return True

        if self.preedit_string:
            if keyval in (IBus.Return, IBus.KP_Enter):
                if self.lookup_table.get_number_of_candidates() > 0:
                    self.commit_candidate()
                else:
                    self.commit_string(self.preedit_string)
                return True
            elif keyval == IBus.Escape:
                self.preedit_string = ''
                self.update_candidates()
                return True
            elif keyval == IBus.BackSpace:
                # サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
                self.preedit_string = re.sub('(?:z[hjkl.-]|[kstnhmyrwgzjdbp]?[aiueo]|.)$', '',
                                             self.preedit_string)
                self.invalidate()
                return True
            elif keyval in num_keys:
                index = num_keys.index(keyval)
                if self.set_lookup_table_cursor_pos_in_current_page(index):
                    self.commit_candidate()
                    return True
                return False
            elif keyval in numpad_keys:
                index = numpad_keys.index(keyval)
                if self.set_lookup_table_cursor_pos_in_current_page(index):
                    self.commit_candidate()
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
                self.cursor_right()
                return True
            elif keyval in (IBus.Left, IBus.KP_Left):
                self.cursor_left()
                return True

        if keyval == IBus.space:
            if len(self.preedit_string) == 0:
                # もし、まだなにもはいっていなければ、ただの空白をそのままいれる。
                return False
            else:
                self.logger.debug("cursor down")
                self.cursor_down()
                return True

        if self.mode == MODE_KANA:
            # Allow typing all ASCII letters and punctuation, except digits
            if ord('!') <= keyval < ord('0') or \
                    ord('9') < keyval <= ord('~'):
                if state & (IBus.ModifierType.CONTROL_MASK | IBus.ModifierType.MOD1_MASK) == 0:
                    if self.cursor_moved:
                        self.commit_candidate()
                    self.preedit_string += chr(keyval)
                    self.invalidate()
                    return True
            else:
                if keyval < 128 and self.preedit_string:
                    self.commit_string(self.preedit_string)
        else:
            return False

        return False

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
        if self.lookup_table.cursor_down():
            self.node_selected[self.current_clause] = self.lookup_table.get_cursor_pos()
            self.cursor_moved = True
            self.refresh()
            return True
        return False

    # 選択する分節を右にずらす。
    def cursor_right(self):
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

        # 選択肢テーブルをアップデートする。
        self.lookup_table.clear()
        for node in self.clauses[self.current_clause]:
            candidate = IBus.Text.new_from_string(node.word)
            self.lookup_table.append_candidate(candidate)
        self.logger.info(f"right cursor：updated lookup table {self.current_clause}")

        self.refresh()

        return True

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

        # 選択肢テーブルをアップデートする。
        self.lookup_table.clear()
        for node in self.clauses[self.current_clause]:
            candidate = IBus.Text.new_from_string(node.word)
            self.lookup_table.append_candidate(candidate)
        self.logger.info(f"left cursor：updated lookup table {self.current_clause}")

        self.refresh()

        return True

    def commit_string(self, text):
        self.cursor_moved = False
        ## TODO ここ変えないとダメ
        self.user_dict.add_entry(self.preedit_string, text)
        self.commit_text(IBus.Text.new_from_string(text))
        self.preedit_string = ''
        self.current_clause = 0
        self.node_selected = {}
        self.update_candidates()

    def build_string(self):
        result = ''
        for clauseid, nodes in enumerate(self.clauses):
            result += nodes[self.node_selected.get(clauseid, 0)].word
        return result

    def commit_candidate(self):
        s = self.build_string()
        self.logger.info("Committing {s}")
        self.commit_string(s)

    # cursor_pos = self.lookup_table.get_cursor_pos()
    # if cursor_pos < len(self.clauses[self.current_clause]):
    #     self.commit_string(self.candidates[cursor_pos])
    # else:
    #     # maybe, not happen, but happen.. why?
    #     self.logger.error(
    #         f"commit_candidate failure: cursor_pos={cursor_pos}, candidates={self.clauses}")
    #     self.commit_string('')

    def update_candidates(self):
        try:
            self._update_candidates()
        except:
            self.logger.error(f"cannot get kanji candidates {sys.exc_info()[0]}", exc_info=True)

    def _update_candidates(self):
        self.lookup_table.clear()
        # self.candidates = []

        if len(self.preedit_string) > 0:
            self.clauses: List[List[Node]] = self.comb.convert2(self.preedit_string)
            # self.logger.debug(f"HAHAHA {str(self.preedit_string)}, {str(self.clauses)}")
            for node in self.clauses[0]:
                candidate = IBus.Text.new_from_string(node.word)
                # self.candidates.append(node.word)
                self.lookup_table.append_candidate(candidate)

        self.refresh()

    def refresh(self):
        attrs = IBus.AttrList()
        preedit_len = len(self.preedit_string)
        first_candidate = self.build_string() if len(self.clauses) > 0 else self.preedit_string

        # にほんご ですね.
        text = IBus.Text.new_from_string(first_candidate)
        text.set_attributes(attrs)
        self.update_auxiliary_text(text, preedit_len > 0)

        attrs.append(IBus.Attribute.new(IBus.AttrType.UNDERLINE,
                                        IBus.AttrUnderline.SINGLE, 0, preedit_len))
        text = IBus.Text.new_from_string(first_candidate)
        text.set_attributes(attrs)

        self.update_preedit_text(text, preedit_len, preedit_len > 0)

        # 候補があれば、選択肢を表示させる。
        self._update_lookup_table()
        self.is_invalidate = False

    def _update_lookup_table(self):
        visible = self.lookup_table.get_number_of_candidates() > 0
        self.update_lookup_table(self.lookup_table, visible)

    def do_focus_in(self):
        self.logger.debug("focus_in")
        self.register_properties(self.prop_list)

    def do_focus_out(self):
        self.logger.debug("focus_out")
        self.do_reset()

    def do_reset(self):
        self.logger.debug("reset")
        self.preedit_string = ''

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
