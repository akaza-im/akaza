use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::ops::Range;

use anyhow::Result;
use log::{error, info, trace, warn};

use ibus_sys::attr_list::{ibus_attr_list_append, ibus_attr_list_new};
use ibus_sys::attribute::{
    ibus_attribute_new, IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE, IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
};
use ibus_sys::core::{
    to_gboolean, IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_HYPER_MASK,
    IBusModifierType_IBUS_META_MASK, IBusModifierType_IBUS_MOD1_MASK,
    IBusModifierType_IBUS_MOD2_MASK, IBusModifierType_IBUS_MOD3_MASK,
    IBusModifierType_IBUS_MOD4_MASK, IBusModifierType_IBUS_MOD5_MASK,
    IBusModifierType_IBUS_RELEASE_MASK, IBusModifierType_IBUS_SHIFT_MASK,
};
use ibus_sys::engine::{
    ibus_engine_commit_text, ibus_engine_hide_preedit_text, ibus_engine_update_auxiliary_text,
    ibus_engine_update_lookup_table, ibus_engine_update_preedit_text, IBusEngine,
};
use ibus_sys::engine::{ibus_engine_hide_auxiliary_text, ibus_engine_hide_lookup_table};
use ibus_sys::glib::gchar;
use ibus_sys::glib::{gboolean, guint};
use ibus_sys::lookup_table::{ibus_lookup_table_append_candidate, IBusLookupTable};
use ibus_sys::text::{ibus_text_new_from_string, ibus_text_set_attributes, StringExt};
use libakaza::akaza_builder::Akaza;
use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::graph_resolver::Candidate;
use libakaza::romkan::RomKanConverter;

use crate::commands::{ibus_akaza_commands_map, IbusAkazaCommand};
use crate::keymap::KeyMap;

#[repr(C)]
#[derive(Debug)]
pub(crate) enum InputMode {
    Hiragana,
    Alnum,
}

#[derive(Debug, Hash, PartialEq, Copy, Clone)]
pub enum KeyState {
    // 何も入力されていない状態。
    PreComposition,
    // 変換処理に入る前。ひらがなを入力している段階。
    Composition,
    // 変換中
    Conversion,
}

#[repr(C)]
pub struct AkazaContext {
    pub(crate) input_mode: InputMode,
    pub(crate) cursor_pos: i32,
    pub(crate) preedit: String,
    pub(crate) lookup_table: IBusLookupTable,
    pub(crate) romkan: RomKanConverter,
    command_map: HashMap<&'static str, IbusAkazaCommand>,
    akaza: Akaza,
    clauses: Vec<VecDeque<Candidate>>,
    // げんざいせんたくされているぶんせつ。
    current_clause: usize,
    is_invalidate: bool,
    cursor_moved: bool,
    // key は、clause 番号。value は、node の index。
    node_selected: HashMap<usize, usize>,
    keymap: KeyMap,
    /// シフト+右 or シフト+左で
    force_selected_clause: Vec<Range<usize>>,
}

impl AkazaContext {
    pub(crate) fn new(akaza: Akaza) -> Self {
        AkazaContext {
            input_mode: InputMode::Hiragana,
            cursor_pos: 0,
            preedit: String::new(),
            //         self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
            lookup_table: IBusLookupTable::new(10, 0, 1, 1),
            romkan: RomKanConverter::default(), // TODO make it configurable.
            command_map: ibus_akaza_commands_map(),
            akaza,
            clauses: vec![],
            current_clause: 0,
            is_invalidate: false,
            cursor_moved: false,
            node_selected: HashMap::new(),
            keymap: KeyMap::new(),
            force_selected_clause: Vec::new(),
        }
    }
}

impl AkazaContext {
    pub fn process_key_event(
        &mut self,
        engine: *mut IBusEngine,
        keyval: guint,
        keycode: guint,
        modifiers: guint,
    ) -> bool {
        trace!(
            "process_key_event: keyval={}, keycode={}, modifiers={}",
            keyval,
            keycode,
            modifiers
        );

        // ignore key release event
        if modifiers & IBusModifierType_IBUS_RELEASE_MASK != 0 {
            return false;
        }
        // keymap.register([KEY_STATE_COMPOSITION], ['Return', 'KP_Enter'], 'commit_preedit')
        let key_state = self.get_key_state();

        // TODO configure keymap in ~/.config/akaza/keymap.yml?
        trace!("KeyState={:?}", key_state);
        if let Some(callback) = self
            .keymap
            .get(
                &key_state,
                keyval,
                modifiers
                    & (IBusModifierType_IBUS_CONTROL_MASK
                        | IBusModifierType_IBUS_SHIFT_MASK
                        | IBusModifierType_IBUS_META_MASK
                        | IBusModifierType_IBUS_HYPER_MASK
                        | IBusModifierType_IBUS_MOD1_MASK
                        | IBusModifierType_IBUS_MOD2_MASK
                        | IBusModifierType_IBUS_MOD3_MASK
                        | IBusModifierType_IBUS_MOD4_MASK
                        | IBusModifierType_IBUS_MOD5_MASK),
            )
            .cloned()
        {
            return self.run_callback_by_name(engine, callback.as_str());
        }

        match &self.input_mode {
            InputMode::Hiragana => {
                if modifiers
                    & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                    != 0
                {
                    return false;
                }

                if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                    trace!("Insert new character to preedit: '{}'", self.preedit);
                    if self.lookup_table.get_number_of_candidates() > 0 {
                        // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                        self.commit_candidate(engine);
                    }

                    // Append the character to preedit string.
                    self.preedit.push(char::from_u32(keyval).unwrap());
                    self.cursor_pos += 1;

                    // And update the display status.
                    self.update_preedit_text_before_henkan(engine);
                    return true;
                }
            }
            InputMode::Alnum => return false,
            // _ => return false,
        }

        false // not proceeded
    }

    pub(crate) fn erase_character_before_cursor(&mut self, engine: *mut IBusEngine) {
        unsafe {
            if self.in_henkan_mode() {
                // 変換中の場合、無変換モードにもどす。
                self.lookup_table.clear();
                ibus_engine_hide_auxiliary_text(engine);
                ibus_engine_hide_lookup_table(engine);
            } else {
                // サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
                self.preedit = self.romkan.remove_last_char(&self.preedit)
            }
            // 変換していないときのレンダリングをする。
            self.update_preedit_text_before_henkan(engine);
        }
    }

    pub(crate) fn update_preedit_text_before_henkan(&mut self, engine: *mut IBusEngine) {
        unsafe {
            if self.preedit.is_empty() {
                ibus_engine_hide_preedit_text(engine);
                return;
            }

            // Convert to Hiragana.
            let (_yomi, surface) = self.make_preedit_word();

            let preedit_attrs = ibus_attr_list_new();
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
                    0,
                    surface.len() as guint,
                ),
            );
            let word_c_str = CString::new(surface.clone()).unwrap();
            let preedit_text = ibus_text_new_from_string(word_c_str.as_ptr() as *const gchar);
            ibus_text_set_attributes(preedit_text, preedit_attrs);
            ibus_engine_update_preedit_text(
                engine,
                preedit_text,
                surface.len() as guint,
                !surface.is_empty() as gboolean,
            )
        }

        /*
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
        */
    }
}

impl Drop for AkazaContext {
    fn drop(&mut self) {
        warn!("Dropping AkazaContext");
    }
}

impl AkazaContext {
    /**
     * 入力モードの変更
     */
    pub(crate) fn set_input_mode(&mut self, input_mode: InputMode, engine: *mut IBusEngine) {
        info!("Changing input mode to : {:?}", input_mode);

        // 変換候補をいったんコミットする。
        self.commit_candidate(engine);

        // TODO update menu prop

        self.input_mode = input_mode;

        /*
        def _set_input_mode(self, mode: InputMode):
            """

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
             */
    }

    pub(crate) fn run_callback_by_name(
        &mut self,
        engine: *mut IBusEngine,
        function_name: &str,
    ) -> bool {
        if let Some(function) = self.command_map.get(function_name) {
            info!("Calling function '{}'", function_name);
            function(self, engine);
            true
        } else {
            error!("Unknown function '{}'", function_name);
            false
        }
    }

    pub(crate) fn get_key_state(&self) -> KeyState {
        // キー入力状態を返す。
        if self.preedit.is_empty() {
            // 未入力状態。
            KeyState::PreComposition
        } else if self.in_henkan_mode() {
            KeyState::Conversion
        } else {
            KeyState::Composition
        }
    }

    pub fn in_henkan_mode(&self) -> bool {
        self.lookup_table.get_number_of_candidates() > 0
    }

    pub fn commit_string(&mut self, engine: *mut IBusEngine, text: &str) {
        unsafe {
            self.cursor_moved = false;

            if self.in_henkan_mode() {
                // 変換モードのときのみ学習を実施する
                let mut targets: Vec<String> = Vec::new();
                for (i, nodes) in self.clauses.iter().enumerate() {
                    let idx = self.node_selected.get(&i).unwrap_or(&0);
                    let node = &nodes[*idx];
                    targets.push((node.kanji.to_string()) + "/" + &node.yomi);
                }
                self.akaza.learn(&targets);
            }

            ibus_engine_commit_text(engine, text.to_ibus_text());

            self.preedit.clear();
            self.clauses.clear();
            self.current_clause = 0;
            self.node_selected.clear();
            self.force_selected_clause.clear();

            self.lookup_table.clear();
            self._update_lookup_table(engine);

            ibus_engine_hide_auxiliary_text(engine);
            ibus_engine_hide_preedit_text(engine);
        }

        /*
        def commit_string(self, text):
            self.logger.info("commit_string.")

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
             */
    }

    pub fn commit_candidate(&mut self, engine: *mut IBusEngine) {
        let s = self.build_string();
        self.commit_string(engine, s.as_str());
    }

    pub(crate) fn build_string(&self) -> String {
        let mut result = String::new();
        for (clauseid, nodes) in self.clauses.iter().enumerate() {
            // TODO lisp をひょうかする
            let idex = if let Some(i) = self.node_selected.get(&clauseid) {
                *i
            } else {
                0
            };
            result += &nodes[idex].kanji;
        }
        result
    }

    pub(crate) fn update_candidates(&mut self, engine: *mut IBusEngine) {
        self._update_candidates(engine).unwrap();
        self.current_clause = 0;
        self.node_selected.clear();
    }

    fn _update_candidates(&mut self, engine: *mut IBusEngine) -> Result<()> {
        if self.preedit.is_empty() {
            self.clauses = vec![]
        } else {
            self.clauses = self
                .akaza
                .convert(self.preedit.as_str(), &self.force_selected_clause)?;

            // [a][bc]
            //    ^^^^
            // 上記の様にフォーカスが当たっている時に extend_clause_left した場合
            // 文節の数がもとより減ることがある。その場合は index error になってしまうので、
            // current_clause を動かす。
            if self.current_clause >= self.clauses.len() {
                self.current_clause = self.clauses.len() - 1;
            }
        }
        self.create_lookup_table();
        self.refresh(engine);
        Ok(())
    }

    /**
     * 現在の候補選択状態から、 lookup table を構築する。
     */
    fn create_lookup_table(&mut self) {
        unsafe {
            // 一旦、ルックアップテーブルをクリアする
            self.lookup_table.clear();

            // 現在の未変換情報を元に、候補を算出していく。
            if !self.clauses.is_empty() {
                // lookup table に候補を詰め込んでいく。
                for node in &self.clauses[self.current_clause] {
                    // TODO lisp
                    let candidate = &node.kanji;
                    ibus_lookup_table_append_candidate(
                        &mut self.lookup_table as *mut _,
                        candidate.to_ibus_text(),
                    );
                }
            }
        }
    }

    fn refresh(&mut self, engine: *mut IBusEngine) {
        unsafe {
            if self.clauses.is_empty() {
                ibus_engine_hide_auxiliary_text(engine);
                ibus_engine_hide_lookup_table(engine);
                ibus_engine_hide_preedit_text(engine);
                return;
            }

            let current_clause = &self.clauses[self.current_clause];
            let current_node = &(current_clause[0]);

            // -- auxiliary text(ポップアップしてるやつのほう)
            let first_candidate = &(current_node.yomi);
            let auxiliary_text = first_candidate.as_str().to_ibus_text();
            ibus_text_set_attributes(auxiliary_text, ibus_attr_list_new());
            ibus_engine_update_auxiliary_text(
                engine,
                auxiliary_text,
                to_gboolean(!self.preedit.is_empty()),
            );

            let text = self.build_string();
            let preedit_attrs = ibus_attr_list_new();
            // 全部に下線をひく。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
                    0,
                    text.len() as guint,
                ),
            );
            let bgstart: u32 = self.clauses.iter().map(|c| (c[0].kanji).len() as u32).sum();
            // 背景色を設定する。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
                    0x00333333,
                    bgstart,
                    bgstart + (current_node.kanji.len() as u32),
                ),
            );
            let preedit_text = text.to_ibus_text();
            ibus_text_set_attributes(preedit_text, preedit_attrs);
            ibus_engine_update_preedit_text(
                engine,
                preedit_text,
                text.len() as guint,
                to_gboolean(!text.is_empty()),
            );

            // 候補があれば、選択肢を表示させる。
            self._update_lookup_table(engine);
            self.is_invalidate = false;
        }
    }

    /// 候補があれば lookup table を表示。なければ非表示にする。
    fn _update_lookup_table(&mut self, engine: *mut IBusEngine) {
        unsafe {
            let visible = self.lookup_table.get_number_of_candidates() > 0;
            ibus_engine_update_lookup_table(
                engine,
                &mut self.lookup_table as *mut _,
                to_gboolean(visible),
            );
        }
    }

    /// (yomi, surface)
    pub fn make_preedit_word(&self) -> (String, String) {
        let mut preedit = self.preedit.clone();
        // 先頭文字が大文字な場合は、そのまま返す。
        // "IME" などと入力された場合は、それをそのまま返すようにする。
        if !preedit.is_empty() && preedit.chars().next().unwrap().is_ascii_uppercase() {
            return (preedit.clone(), preedit);
        }

        // hogen と入力された場合、"ほげn" と表示する。
        // hogena となったら "ほげな"
        // hogenn となったら "ほげん" と表示する必要があるため。
        // 「ん」と一旦表示された後に「な」に変化したりすると気持ち悪く感じる。
        let suffix = if preedit.ends_with('n') && !preedit.ends_with("nn") {
            let (i, _) = preedit.char_indices().last().unwrap();
            preedit = preedit[0..i].to_string();
            "n"
        } else {
            ""
        };
        let yomi = self.romkan.to_hiragana(preedit.as_str());
        let surface = yomi.clone();
        (yomi + suffix, surface + suffix)

        /*
            yomi = self.romkan.to_hiragana(self.preedit_string)
            if self.input_mode == INPUT_MODE_KATAKANA:
                return yomi, jaconv.hira2kata(yomi)
            elif self.input_mode == INPUT_MODE_HALFWIDTH_KATAKANA:
                return yomi, jaconv.z2h(jaconv.hira2kata(yomi))
            else:
                return yomi, yomi
        */
    }

    /// 前の変換候補を選択する。
    pub(crate) fn cursor_up(&mut self, engine: *mut IBusEngine) {
        if self.lookup_table.cursor_up() {
            self.node_selected.insert(
                self.current_clause,
                self.lookup_table.get_cursor_pos() as usize,
            );
            self.cursor_moved = true;
            self.refresh(engine);
        }
    }

    /// 次の変換候補を選択する。
    pub fn cursor_down(&mut self, engine: *mut IBusEngine) {
        if self.lookup_table.cursor_down() {
            self.node_selected.insert(
                self.current_clause,
                self.lookup_table.get_cursor_pos() as usize,
            );
            self.cursor_moved = true;
            self.refresh(engine);
        }
    }

    /// 選択する分節を右にずらす。
    pub(crate) fn cursor_right(&mut self, engine: *mut IBusEngine) {
        // 分節がない場合は、何もしない。
        if self.clauses.is_empty() {
            return;
        }

        // 既に一番右だった場合、一番左にいく。
        if self.current_clause == self.clauses.len() - 1 {
            self.current_clause = 0;
        } else {
            self.current_clause += 1;
        }

        self.cursor_moved = true;
        self.create_lookup_table();

        self.refresh(engine);
    }

    /// 選択する分節を左にずらす。
    pub(crate) fn cursor_left(&mut self, engine: *mut IBusEngine) {
        // 分節がなければ何もしない
        if self.clauses.is_empty() {
            return;
        }

        // 既に一番左だった場合、一番右にいく
        if self.current_clause == 0 {
            self.current_clause = self.clauses.len() - 1
        } else {
            self.current_clause -= 1
        }

        self.cursor_moved = true;
        self.create_lookup_table();

        self.refresh(engine);
    }

    /// 文節の選択範囲を右方向に広げる
    pub fn extend_clause_right(&mut self, engine: *mut IBusEngine) -> Result<()> {
        self.force_selected_clause = extend_right(&self.clauses, self.current_clause);
        self._update_candidates(engine)?;
        self.node_selected.clear();
        Ok(())
    }

    /// 文節の選択範囲を左方向に広げる
    pub fn extend_clause_left(&mut self, engine: *mut IBusEngine) -> Result<()> {
        self.force_selected_clause = extend_left(&self.clauses, self.current_clause);

        self._update_candidates(engine)?;
        self.node_selected.clear();
        Ok(())
    }
}
