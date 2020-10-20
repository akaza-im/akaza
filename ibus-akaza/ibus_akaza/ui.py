import time
from typing import List, Dict, Optional

import gi

gi.require_version('GLib', '2.0')
gi.require_version('IBus', '1.0')

from gi.repository import IBus
from gi.repository import GLib

import sys
import logging
import pathlib
import threading
import gettext

from jaconv import jaconv

from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel, \
    Slice, create_node, TinyLisp, build_romkan_converter

from ibus_akaza import config_loader
from ibus_akaza.config import MODEL_DIR

from .keymap import build_default_keymap, KEY_STATE_PRECOMPOSITION, KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION
from .input_mode import get_input_mode_from_prop_name, InputMode, INPUT_MODE_ALNUM, INPUT_MODE_HIRAGANA, \
    get_all_input_modes, INPUT_MODE_FULLWIDTH_ALNUM, INPUT_MODE_KATAKANA, INPUT_MODE_HALFWIDTH_KATAKANA

_ = lambda a: gettext.dgettext('ibus-akaza', a)


def build_akaza():
    configdir = pathlib.Path(GLib.get_user_config_dir(), 'ibus-akaza')

    user_settings = config_loader.ConfigLoader()
    user_dicts = list(user_settings.load_user_dict())

    user_language_model_path = configdir.joinpath('user_language_model')
    user_language_model_path.mkdir(parents=True, exist_ok=True, mode=0o700)
    user_language_model = UserLanguageModel(
        str(user_language_model_path.joinpath('unigram.txt')),
        str(user_language_model_path.joinpath('bigram.txt'))
    )

    system_dict = BinaryDict()
    print(f"{MODEL_DIR + '/system_dict.trie'}")
    system_dict.load(MODEL_DIR + "/system_dict.trie")

    system_unigram_lm = SystemUnigramLM()
    system_unigram_lm.load(MODEL_DIR + "/lm_v2_1gram.trie")

    system_bigram_lm = SystemBigramLM()
    system_bigram_lm.load(MODEL_DIR + "/lm_v2_2gram.trie")

    emoji_dict = BinaryDict()
    emoji_dict.load(MODEL_DIR + "/single_term.trie")

    resolver = GraphResolver(
        user_language_model,
        system_unigram_lm,
        system_bigram_lm,
        [system_dict] + user_dicts,
        [emoji_dict],
    )

    additional = user_settings.get('romaji')
    if additional is None:
        additional = {}
    romkan = build_romkan_converter(additional)

    lisp_evaluator = TinyLisp()

    return user_language_model, Akaza(resolver, romkan), romkan, lisp_evaluator, user_settings, system_unigram_lm


try:
    t0 = time.time()

    user_language_model, akaza, romkan, lisp_evaluator, user_settings, system_unigram_lm = build_akaza()

    def save_periodically():
        while True:
            user_language_model.save()
            time.sleep(10)

    user_language_model_save_thread = threading.Thread(
        name='user_language_model_save_thread',
        target=lambda: save_periodically(),
        daemon=True,
    )
    user_language_model_save_thread.start()

    keymap = build_default_keymap()

    logging.info(f"Loaded Akaza in {time.time() - t0} seconds")
except:
    logging.error("Cannot initialize Akaza.", exc_info=True)
    sys.exit(1)


# ----------------------------------------------------------------------
# the engine
# ----------------------------------------------------------------------

class AkazaIBusEngine(IBus.Engine):
    input_mode_prop: IBus.Property
    user_language_model: UserLanguageModel
    current_clause: int
    node_selected: Dict[int, int]
    clauses: List[List[Node]]
    prop_list: IBus.PropList
    akaza: Akaza
    input_mode: InputMode
    force_selected_clause: Optional[List[slice]]

    __gtype_name__ = 'AkazaIBusEngine'

    def __init__(self):
        super(AkazaIBusEngine, self).__init__()
        self.is_invalidate = False
        # 未確定文字列。
        self.preedit_string = ''
        # 候補文字列
        self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
        self.akaza = akaza
        self.user_language_model = user_language_model
        self.logger = logging.getLogger(__name__)
        self.input_mode = INPUT_MODE_HIRAGANA

        # 変換候補。文節ごとの配列。
        self.clauses = []
        # 現在選択されている、文節。
        self.current_clause = 0
        # key は、clause 番号。value は、node の index。
        self.node_selected = {}

        # 文節を選びなおしたもの。
        self.force_selected_clause = None

        # カーソル変更をしたばっかりかどうかを、みるフラグ。
        self.cursor_moved = False

        self.romkan = romkan

        # タスクメニューからポップアップで選べるメニューについて、セットアップする。
        self.__prop_dict = {}
        self.prop_list = self._init_props()

        self.live_conversion_mode = user_settings.get('live_conversion', False)

        self.logger.debug("Create Akaza engine OK: 20200916")

    def _init_props(self) -> IBus.PropList:
        """
        タスクメニューからポップアップして選べるメニューを構築する。
        """
        prop_list = IBus.PropList()
        self.input_mode_prop = IBus.Property(key=u'InputMode',
                                             prop_type=IBus.PropType.MENU,
                                             label=IBus.Text.new_from_string(_("Input mode (%s)") % 'あ'),
                                             icon='',
                                             tooltip=IBus.Text.new_from_string(_("Switch input mode")),
                                             sensitive=True,
                                             visible=True,
                                             state=IBus.PropState.UNCHECKED,
                                             sub_props=None)
        prop_list.append(self.input_mode_prop)

        props = IBus.PropList()
        for input_mode in get_all_input_modes():
            props.append(IBus.Property(key=input_mode.prop_name,
                                       prop_type=IBus.PropType.RADIO,
                                       label=IBus.Text.new_from_string(input_mode.label),
                                       icon=None,
                                       tooltip=None,
                                       sensitive=True,
                                       visible=True,
                                       state=IBus.PropState.UNCHECKED,
                                       sub_props=None))
        i = 0
        while props.get(i) is not None:
            prop = props.get(i)
            self.__prop_dict[prop.get_key()] = prop
            i += 1
        props.get(self.input_mode.mode_code).set_state(IBus.PropState.CHECKED)
        self.input_mode_prop.set_sub_props(props)

        return prop_list

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
        self.logger.info("do_candidate_clicked")
        if self.set_lookup_table_cursor_pos_in_current_page(index):
            self.commit_candidate()

    def do_process_key_event(self, keyval, keycode, state):
        try:
            return self._do_process_key_event(keyval, keycode, state)
        except:
            self.logger.error(f"Cannot process key event: keyval={keyval} keycode={keycode} state={state}",
                              exc_info=True)
            return False

    def _get_key_state(self):
        """
        キー入力状態を返す。
        """
        if len(self.preedit_string) == 0:
            # 未入力
            self.logger.debug("key_state: KEY_STATE_PRECOMPOSITION")
            return KEY_STATE_PRECOMPOSITION
        else:
            if self.in_henkan_mode():
                # 変換中
                self.logger.debug("key_state: KEY_STATE_CONVERSION")
                return KEY_STATE_CONVERSION
            else:
                # 入力されているがまだ変換されていない
                self.logger.debug("key_state: KEY_STATE_COMPOSITION")
                return KEY_STATE_COMPOSITION

    def _do_process_key_event(self, keyval, keycode, state):
        self.logger.debug(
            "process_key_event(%s=%04x, %04x, %04x)" % (IBus.keyval_name(keyval), keyval, keycode, state))

        # ignore key release events
        is_press = ((state & IBus.ModifierType.RELEASE_MASK) == 0)
        if not is_press:
            return False

        got_method = keymap.get(self._get_key_state(), keyval, state)
        if got_method is not None:
            self.logger.debug(f"Calling method: {got_method}")
            getattr(self, got_method)()
            return True

        if self.input_mode in (INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA, INPUT_MODE_HALFWIDTH_KATAKANA):
            # Allow typing all ASCII letters and punctuation
            if ord('!') <= keyval <= ord('~'):
                if state & (IBus.ModifierType.CONTROL_MASK | IBus.ModifierType.MOD1_MASK) == 0:
                    if self.live_conversion_mode:
                        self.preedit_string += chr(keyval)
                        # この時点では、preedit string にだけ、追加して表示するひつようがあります。
                        self.update_candidates()
                        return True
                    else:
                        if self.in_henkan_mode():
                            self.logger.info("Call `commit_candidate()` since it's in henkan mode...")
                            self.commit_candidate()

                        self.preedit_string += chr(keyval)
                        # この時点では、preedit string にだけ、追加して表示する必要があります。
                        self.update_preedit_text_before_henkan()
                        return True
            else:
                if keyval < 128 and self.preedit_string:
                    self.commit_string(self.preedit_string)
        elif self.input_mode == INPUT_MODE_FULLWIDTH_ALNUM:
            self.logger.info("In full-width alnum mode.")
            if ord('!') <= keyval <= ord('~'):
                if state & (IBus.ModifierType.CONTROL_MASK | IBus.ModifierType.MOD1_MASK) == 0:
                    self.commit_text(IBus.Text.new_from_string(jaconv.h2z(chr(keyval), ascii=True, digit=True)))
                    return True
        else:
            return False

        return False

    def _set_input_mode(self, mode: InputMode):
        """
        入力モードの変更
        """
        self.logger.info(f"input mode activate: {mode}")

        # 変換候補をいったんコミットする。
        self.commit_candidate()

        label = _("Input mode (%s)") % mode.symbol
        prop = self.input_mode_prop
        prop.set_symbol(IBus.Text.new_from_string(mode.symbol))
        prop.set_label(IBus.Text.new_from_string(label))
        self.update_property(prop)

        self.__prop_dict[mode.prop_name].set_state(IBus.PropState.CHECKED)
        self.update_property(self.__prop_dict[mode.prop_name])

        self.input_mode = mode

    def set_input_mode_hiragana(self):
        self._set_input_mode(INPUT_MODE_HIRAGANA)

    def set_input_mode_katakana(self):
        self._set_input_mode(INPUT_MODE_KATAKANA)

    def set_input_mode_alnum(self):
        self._set_input_mode(INPUT_MODE_ALNUM)

    def set_input_mode_fullwidth_alnum(self):
        self._set_input_mode(INPUT_MODE_FULLWIDTH_ALNUM)

    def do_property_activate(self, prop_name, state):
        """
        Set props
        """
        self.logger.debug(f"PropertyActivate(prop_name={prop_name}, state={state})")
        if state == IBus.PropState.CHECKED:
            if prop_name is None:
                return
            elif prop_name.startswith(u'InputMode.'):
                self.__input_mode_activate(prop_name, state)
                return

    def __input_mode_activate(self, prop_name, state):
        input_mode = get_input_mode_from_prop_name(prop_name)
        if input_mode is None:
            self.logger.error(f'Unknown prop_name = {prop_name}')
            return
        self._set_input_mode(input_mode)

        # self.__reset()
        # self.__invalidate()

    def in_henkan_mode(self):
        return self.lookup_table.get_number_of_candidates() > 0

    def convert_to_full_katakana(self):
        """
        convert to full-width katakana (standard katakana): ほわいと → ホワイト
        """
        self.logger.info("Convert to full katakana")

        # カタカナ候補のみを表示するようにする。
        hira = self.romkan.to_hiragana(self.preedit_string)
        kata = jaconv.hira2kata(hira)

        self.convert_to_single(hira, kata)

    def convert_to_full_hiragana(self):
        """
        convert selected word/characters to full-width hiragana (standard hiragana): ホワイト → ほわいと
        """
        self.logger.info("Convert to full hiragana")

        # カタカナ候補のみを表示するようにする。
        hira = self.romkan.to_hiragana(self.preedit_string)
        self.convert_to_single(hira, hira)

    def convert_to_half_katakana(self):
        """
        convert to half-width katakana (katakana for specific purpose): ホワイト → ﾎﾜｲﾄ
        """
        self.logger.info("Convert to half katakana")

        # 半角カタカナ候補のみを表示するようにする。
        hira = self.romkan.to_hiragana(self.preedit_string)
        kata = jaconv.hira2kata(hira)
        kata = jaconv.z2h(kata)

        self.convert_to_single(hira, kata)

    def convert_to_half_romaji(self):
        """
        convert to half-width romaji, all-capitals, proper noun capitalization (latin script like
        standard English): ホワイト → howaito → HOWAITO → Howaito
        """
        self.logger.info("Convert to half romaji")

        # 半角カタカナ候補のみを表示するようにする。
        hira = self.romkan.to_hiragana(self.preedit_string)
        romaji = jaconv.z2h(self.preedit_string)

        self.convert_to_single(hira, romaji)

    def convert_to_full_romaji(self):
        """
        convert to full-width romaji, all-capitals, proper noun capitalization (latin script inside
        Japanese text): ホワイト → ｈｏｗａｉｔｏ → ＨＯＷＡＩＴＯ → Ｈｏｗａｉｔｏ
        """
        self.logger.info("Convert to full romaji")

        hira = self.romkan.to_hiragana(self.preedit_string)
        romaji = jaconv.h2z(self.preedit_string, kana=True, digit=True, ascii=True)

        self.convert_to_single(hira, romaji)

    def convert_to_single(self, yomi, word) -> None:
        """
        特定の1文節の文章を候補として表示する。
        F6 などを押した時用。
        """
        # 候補を設定
        self.clauses = [[create_node(system_unigram_lm, 0, yomi, word)]]
        self.current_clause = 0
        self.node_selected = {}
        self.force_selected_clause = None

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

        max_len = max([clause[0].get_start_pos() + len(clause[0].get_yomi()) for clause in self.clauses])

        self.force_selected_clause = []
        for i, clause in enumerate(self.clauses):
            node = clause[0]
            if self.current_clause == i:
                # 現在選択中の文節の場合、伸ばす。
                self.force_selected_clause.append(
                    slice(node.get_start_pos(), min(node.get_start_pos() + len(node.get_yomi()) + 1, max_len)))
            elif self.current_clause + 1 == i:
                # 次の分節を一文字ヘラス
                self.force_selected_clause.append(
                    slice(node.get_start_pos() + 1, node.get_start_pos() + len(node.get_yomi())))
            else:
                # それ以外は現在指定の分節のまま
                self.force_selected_clause.append(
                    slice(node.get_start_pos(), node.get_start_pos() + len(node.get_yomi())))

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
                    slice(node.get_start_pos() - 1, node.get_start_pos() + len(node.get_yomi())))
            elif target_clause - 1 == i:
                # 前の分節を一文字ヘラス
                self.force_selected_clause.append(
                    slice(node.get_start_pos(), node.get_start_pos() + len(node.get_yomi()) - 1))
            else:
                # それ以外は現在指定の分節のまま
                self.force_selected_clause.append(
                    slice(node.get_start_pos(), node.get_start_pos() + len(node.get_yomi())))

        self.force_selected_clause = [x for x in self.force_selected_clause if x.start != x.stop]
        self._update_candidates()
        # TODO: もし、分節の長さをいじったら、self.node_selected も変更するべき。

    def commit_preedit(self):
        # 無変換状態では、ひらがなに変換してコミットします。
        yomi, word = self._make_preedit_word()
        self.commit_string(word)

    def commit_string(self, text):
        self.logger.info("commit_string.")
        self.cursor_moved = False

        if self.in_henkan_mode():
            # 変換モードのときのみ学習を実施する。
            candidate_nodes = []
            for clauseid, nodes in enumerate(self.clauses):
                candidate_nodes.append(nodes[self.node_selected.get(clauseid, 0)])
            self.user_language_model.add_entry(candidate_nodes)

        self.commit_text(IBus.Text.new_from_string(text))

        self.preedit_string = ''
        self.clauses = []
        self.current_clause = 0
        self.node_selected = {}
        self.force_selected_clause = None

        self.lookup_table.clear()
        self.update_lookup_table(self.lookup_table, False)

        self.hide_auxiliary_text()
        self.hide_preedit_text()

    def build_string(self):
        result = ''
        for clauseid, nodes in enumerate(self.clauses):
            result += nodes[self.node_selected.get(clauseid, 0)].surface(lisp_evaluator)
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
            # print(f"-------{self.preedit_string}-----{self.force_selected_clause}----PPP")
            slices = None
            if self.force_selected_clause:
                slices = [Slice(s.start, s.stop-s.start) for s in self.force_selected_clause]
            # print(f"-------{self.preedit_string}-----{self.force_selected_clause}---{slices}----PPP")
            self.clauses = self.akaza.convert(self.preedit_string, slices)
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
        first_candidate = current_node.get_yomi()
        auxiliary_text = IBus.Text.new_from_string(first_candidate)
        auxiliary_text.set_attributes(IBus.AttrList())
        self.update_auxiliary_text(auxiliary_text, preedit_len > 0)

        text = self.build_string()
        preedit_attrs = IBus.AttrList()
        # 全部に下線をひく。
        preedit_attrs.append(IBus.Attribute.new(IBus.AttrType.UNDERLINE,
                                                IBus.AttrUnderline.SINGLE, 0, len(text)))
        bgstart = sum([len(self.clauses[i][0].surface(lisp_evaluator)) for i in range(0, self.current_clause)])
        # 背景色を設定する。
        preedit_attrs.append(IBus.Attribute.new(
            IBus.AttrType.BACKGROUND,
            0x00333333,
            bgstart,
            bgstart + len(current_node.surface(lisp_evaluator))))
        preedit_text = IBus.Text.new_from_string(text)
        preedit_text.set_attributes(preedit_attrs)
        self.update_preedit_text(preedit_text, len(text), len(text) > 0)

        # 候補があれば、選択肢を表示させる。
        self._update_lookup_table()
        self.is_invalidate = False

    def _make_preedit_word(self):
        """
        preedict string をよい感じに見せる。
        """
        self.logger.debug(f"_make_preedit_word: {self.preedit_string}")

        # 先頭が大文字だと、
        if len(self.preedit_string) > 0 and self.preedit_string[0].isupper() \
                and self.force_selected_clause is None:
            return self.preedit_string, self.preedit_string

        yomi = self.romkan.to_hiragana(self.preedit_string)
        if self.input_mode == INPUT_MODE_KATAKANA:
            return yomi, jaconv.hira2kata(yomi)
        elif self.input_mode == INPUT_MODE_HALFWIDTH_KATAKANA:
            return yomi, jaconv.z2h(jaconv.hira2kata(yomi))
        else:
            return yomi, yomi

    def update_preedit_text_before_henkan(self):
        """
        無変換状態で、どんどん入力していくフェーズ。
        """
        self.logger.debug(f"update_preedit_text_before_henkan: {self.preedit_string}")

        if len(self.preedit_string) == 0:
            self.hide_preedit_text()
            return

        # 平仮名にする。
        yomi, word = self._make_preedit_word()
        self.clauses = [
            [create_node(system_unigram_lm, 0, yomi, word)]
        ]
        self.current_clause = 0

        preedit_attrs = IBus.AttrList()
        preedit_attrs.append(IBus.Attribute.new(IBus.AttrType.UNDERLINE,
                                                IBus.AttrUnderline.SINGLE, 0, len(word)))
        preedit_text = IBus.Text.new_from_string(word)
        preedit_text.set_attributes(preedit_attrs)
        self.update_preedit_text(text=preedit_text, cursor_pos=len(word), visible=(len(word) > 0))

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
                candidate = IBus.Text.new_from_string(node.surface(lisp_evaluator))
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
        # フォーカスアウトイベントは、かなりマメに実行される。
        # どういうきっかけで発火しているのかよくわからない。
        self.logger.debug("do_focus_out")
        # self.do_reset()

    def do_reset(self):
        self.logger.debug("do_reset")
        self.preedit_string = ''
        self.force_selected_clause = None
        self.clauses = []
        self.current_clause = 0
        self.node_selected = {}

        self.lookup_table.clear()
        self.hide_auxiliary_text()
        self.hide_lookup_table()

    def do_page_up(self):
        return self.page_up()

    def do_page_down(self):
        return self.page_down()

    def do_cursor_up(self):
        return self.cursor_up()

    def do_cursor_down(self):
        return self.cursor_down()

    def erase_character_before_cursor(self):
        self.logger.info(f"erase_character_before_cursor: {self.preedit_string}")
        if self.in_henkan_mode():
            # 変換中の場合、無変換モードにもどす。
            self.lookup_table.clear()
            self.hide_auxiliary_text()
            self.hide_lookup_table()
        else:
            # サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
            self.preedit_string = self.romkan.remove_last_char(self.preedit_string)
        # 変換していないときのレンダリングをする。
        self.update_preedit_text_before_henkan()

    def escape(self):
        self.logger.info(f"escape: {self.preedit_string}")
        self.preedit_string = ''
        self.update_candidates()


for n in range(0, 10):
    def create_cb(nn):
        idx = 9 if nn == 0 else nn - 1

        def cb(self):
            if self.set_lookup_table_cursor_pos_in_current_page(idx):
                self.refresh()

        return cb


    setattr(AkazaIBusEngine, f"press_number_{n}", create_cb(n))
