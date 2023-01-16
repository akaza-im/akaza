use alloc::collections::vec_deque::VecDeque;
use alloc::ffi::CString;
use std::collections::HashMap;
use std::ops::Range;

use anyhow::Result;
use kelp::{h2z, hira2kata, z2h, ConvOption};
use log::{debug, error, info, trace, warn};

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
    ibus_engine_commit_text, ibus_engine_hide_preedit_text, ibus_engine_register_properties,
    ibus_engine_update_auxiliary_text, ibus_engine_update_lookup_table,
    ibus_engine_update_preedit_text, IBusEngine,
};
use ibus_sys::engine::{ibus_engine_hide_auxiliary_text, ibus_engine_hide_lookup_table};
use ibus_sys::glib::{g_object_ref_sink, gchar, gpointer};
use ibus_sys::glib::{gboolean, guint};
use ibus_sys::lookup_table::IBusLookupTable;
use ibus_sys::prop_list::{ibus_prop_list_append, ibus_prop_list_new, IBusPropList};
use ibus_sys::property::{
    ibus_property_new, ibus_property_set_sub_props, IBusPropState_PROP_STATE_CHECKED,
    IBusPropState_PROP_STATE_UNCHECKED, IBusPropType_PROP_TYPE_MENU, IBusPropType_PROP_TYPE_RADIO,
    IBusProperty,
};
use ibus_sys::text::{ibus_text_new_from_string, ibus_text_set_attributes, IBusText, StringExt};
use libakaza::consonant::ConsonantSuffixExtractor;
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngine;
use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::candidate::Candidate;
use libakaza::lm::system_bigram::MarisaSystemBigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use libakaza::romkan::RomKanConverter;

use crate::commands::{ibus_akaza_commands_map, IbusAkazaCommand};
use crate::input_mode::{
    get_all_input_modes, get_input_mode_from_prop_name, InputMode, INPUT_MODE_HALFWIDTH_KATAKANA,
    INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA,
};
use crate::keymap::KeyMap;

// #[repr(C)]
// #[derive(Debug)]
// pub(crate) enum InputMode {
//     Hiragana,
//     Alnum,
// }

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
    engine: BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM>,
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
    prop_list: *mut IBusPropList,
    pub input_mode_prop: *mut IBusProperty,
    pub prop_dict: HashMap<String, *mut IBusProperty>,
    pub consonant_suffix_extractor: ConsonantSuffixExtractor,
}

impl AkazaContext {
    /// Set props
    pub(crate) fn do_property_activate(
        &mut self,
        engine: *mut IBusEngine,
        prop_name: String,
        prop_state: guint,
    ) {
        debug!("do_property_activate: {}, {}", prop_name, prop_state);
        if prop_state == IBusPropState_PROP_STATE_CHECKED && prop_name.starts_with("InputMode.") {
            self.input_mode_activate(engine, prop_name, prop_state);
        }
    }

    pub fn input_mode_activate(
        &mut self,
        engine: *mut IBusEngine,
        prop_name: String,
        _prop_state: guint,
    ) {
        if let Ok(input_mode) = get_input_mode_from_prop_name(prop_name.as_str()) {
            self.set_input_mode(engine, &input_mode);
        } else {
            warn!("Unknown prop_name: {}", prop_name);
        }
    }
}

impl AkazaContext {
    pub(crate) fn process_num_key(&mut self, nn: i32, engine: *mut IBusEngine) {
        let idx = if nn == 0 { 9 } else { nn - 1 };

        if self.set_lookup_table_cursor_pos_in_current_page(idx) {
            self.refresh(engine)
        }
    }

    /// Sets the cursor in the lookup table to index in the current page
    /// Returns True if successful, False if not.
    fn set_lookup_table_cursor_pos_in_current_page(&mut self, idx: i32) -> bool {
        trace!("set_lookup_table_cursor_pos_in_current_page: {}", idx);

        let page_size = self.lookup_table.get_page_size();
        if idx > (page_size as i32) {
            info!("Index too big: {} > {}", idx, page_size);
            return false;
        }

        let page = self.lookup_table.get_cursor_pos() / page_size;
        // let pos_in_page = self.lookup_table.get_cursor_pos() % page_size;

        let new_pos = page * page_size + (idx as u32);

        if new_pos > self.lookup_table.get_number_of_candidates() {
            info!(
                "new_pos too big: {} > {}",
                new_pos,
                self.lookup_table.get_number_of_candidates()
            );
            return false;
        }
        self.lookup_table.set_cursor_pos(new_pos);
        self.node_selected.insert(
            self.current_clause,
            self.lookup_table.get_cursor_pos() as usize,
        );

        true
    }
}

impl AkazaContext {
    pub(crate) fn new(
        akaza: BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM>,
    ) -> Self {
        let (input_mode_prop, prop_list, prop_dict) = Self::init_props();
        AkazaContext {
            input_mode: INPUT_MODE_HIRAGANA,
            cursor_pos: 0,
            preedit: String::new(),
            //         self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
            lookup_table: IBusLookupTable::new(10, 0, 1, 1),
            romkan: RomKanConverter::default(), // TODO make it configurable.
            command_map: ibus_akaza_commands_map(),
            engine: akaza,
            clauses: vec![],
            current_clause: 0,
            is_invalidate: false,
            cursor_moved: false,
            node_selected: HashMap::new(),
            keymap: KeyMap::new(),
            force_selected_clause: Vec::new(),
            prop_list,
            input_mode_prop,
            prop_dict,
            consonant_suffix_extractor: ConsonantSuffixExtractor::default(),
        }
    }

    /// タスクメニューからポップアップして選べるメニューを構築する。
    pub fn init_props() -> (
        *mut IBusProperty,
        *mut IBusPropList,
        HashMap<String, *mut IBusProperty>,
    ) {
        unsafe {
            let prop_list =
                g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;

            let input_mode_prop = g_object_ref_sink(ibus_property_new(
                "InputMode\0".as_ptr() as *const gchar,
                IBusPropType_PROP_TYPE_MENU,
                "Input mode (あ)".to_ibus_text(),
                "\0".as_ptr() as *const gchar,
                "Switch input mode".to_ibus_text(),
                to_gboolean(true),
                to_gboolean(true),
                IBusPropState_PROP_STATE_UNCHECKED,
                std::ptr::null_mut() as *mut IBusPropList,
            ) as gpointer) as *mut IBusProperty;
            ibus_prop_list_append(prop_list, input_mode_prop);

            let props = g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;
            let mut prop_map: HashMap<String, *mut IBusProperty> = HashMap::new();
            for input_mode in get_all_input_modes() {
                let prop = g_object_ref_sink(ibus_property_new(
                    (input_mode.prop_name.to_string() + "\0").as_ptr() as *const gchar,
                    IBusPropType_PROP_TYPE_RADIO,
                    input_mode.label.to_ibus_text(),
                    "\0".as_ptr() as *const gchar,
                    std::ptr::null_mut() as *mut IBusText,
                    to_gboolean(true),
                    to_gboolean(true),
                    IBusPropState_PROP_STATE_UNCHECKED,
                    std::ptr::null_mut() as *mut IBusPropList,
                ) as gpointer) as *mut IBusProperty;
                prop_map.insert(input_mode.prop_name.to_string(), prop);
                ibus_prop_list_append(props, prop);
            }

            ibus_property_set_sub_props(input_mode_prop, props);

            (input_mode_prop, prop_list, prop_map)
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

        match self.input_mode.prop_name {
            "InputMode.Hiragana" | "InputMode.Katakana" | "InputMode.HalfWidthKatakana" => {
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
            "InputMode.Alphanumeric" => return false,
            "InputMode.FullWidthAlnum" => {
                if ('!' as u32) <= keyval
                    && keyval <= ('~' as u32)
                    && (modifiers
                        & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK))
                        == 0
                {
                    let option = ConvOption {
                        ascii: true,
                        digit: true,
                        ..Default::default()
                    };
                    let text = h2z(char::from_u32(keyval).unwrap().to_string().as_str(), option);
                    unsafe { ibus_engine_commit_text(engine, text.to_ibus_text()) };
                    return true;
                }
            }
            _ => {
                warn!("Unknown prop: {}", self.input_mode.prop_name);
                return false;
            }
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
    pub(crate) fn set_input_mode(&mut self, engine: *mut IBusEngine, input_mode: &InputMode) {
        info!("Changing input mode to : {:?}", input_mode);

        // 変換候補をいったんコミットする。
        self.commit_candidate(engine);

        // TODO update menu prop

        /*
        label = _("Input mode (%s)") % mode.symbol
        prop = self.input_mode_prop
        prop.set_symbol(IBus.Text.new_from_string(mode.symbol))
        prop.set_label(IBus.Text.new_from_string(label))
        self.update_property(prop)

        self.__prop_dict[mode.prop_name].set_state(IBus.PropState.CHECKED)
        self.update_property(self.__prop_dict[mode.prop_name])
         */

        self.input_mode = *input_mode;
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
                let mut targets: Vec<Candidate> = Vec::new();
                for (i, candidates) in self.clauses.iter().enumerate() {
                    let idx = self.node_selected.get(&i).unwrap_or(&0);
                    targets.push(candidates[*idx].clone());
                }
                self.engine.learn(&targets);
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
            let idex = if let Some(i) = self.node_selected.get(&clauseid) {
                *i
            } else {
                0
            };
            result += &nodes[idex].surface_with_dynamic();
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
                .engine
                .convert(self.preedit.as_str(), Some(&self.force_selected_clause))?;

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
        // 一旦、ルックアップテーブルをクリアする
        self.lookup_table.clear();

        // 現在の未変換情報を元に、候補を算出していく。
        if !self.clauses.is_empty() {
            // lookup table に候補を詰め込んでいく。
            for node in &self.clauses[self.current_clause] {
                let candidate = &node.surface_with_dynamic();
                self.lookup_table.append_candidate(candidate.to_ibus_text());
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
            let bgstart: u32 = self
                .clauses
                .iter()
                .map(|c| (c[0].surface).len() as u32)
                .sum();
            // 背景色を設定する。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
                    0x00333333,
                    bgstart,
                    bgstart + (current_node.surface.len() as u32),
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
        let preedit = self.preedit.clone();
        // 先頭文字が大文字な場合は、そのまま返す。
        // "IME" などと入力された場合は、それをそのまま返すようにする。
        if !preedit.is_empty() && preedit.chars().next().unwrap().is_ascii_uppercase() {
            return (preedit.clone(), preedit);
        }

        // hogen と入力された場合、"ほげn" と表示する。
        // hogena となったら "ほげな"
        // hogenn となったら "ほげん" と表示する必要があるため。
        // 「ん」と一旦表示された後に「な」に変化したりすると気持ち悪く感じる。
        let (preedit, suffix) = self.consonant_suffix_extractor.extract(preedit.as_str());

        let yomi = self.romkan.to_hiragana(preedit.as_str());
        let surface = yomi.clone();
        if self.input_mode == INPUT_MODE_KATAKANA {
            (
                yomi.to_string() + suffix.as_str(),
                hira2kata(yomi.as_str(), ConvOption::default()) + suffix.as_str(),
            )
        } else if self.input_mode == INPUT_MODE_HALFWIDTH_KATAKANA {
            (
                yomi.to_string() + suffix.as_str(),
                z2h(
                    hira2kata(yomi.as_str(), ConvOption::default()).as_str(),
                    ConvOption::default(),
                ) + suffix.as_str(),
            )
        } else {
            (yomi + suffix.as_str(), surface + suffix.as_str())
        }

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

    pub fn do_candidate_clicked(
        &mut self,
        engine: *mut IBusEngine,
        index: guint,
        _button: guint,
        _state: guint,
    ) {
        info!("do_candidate_clicked");
        if self.set_lookup_table_cursor_pos_in_current_page(index as i32) {
            self.commit_candidate(engine)
        }
    }

    pub fn do_focus_in(&mut self, engine: *mut IBusEngine) {
        trace!("do_focus_in");
        unsafe {
            ibus_engine_register_properties(engine, self.prop_list);
        }
    }

    /// convert selected word/characters to full-width hiragana (standard hiragana): ホワイト → ほわいと
    pub fn convert_to_full_hiragana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        info!("Convert to full hiragana");
        let hira = self.romkan.to_hiragana(self.preedit.as_str());
        self.convert_to_single(engine, hira.as_str(), hira.as_str())
    }

    /// convert to full-width katakana (standard katakana): ほわいと → ホワイト
    pub fn convert_to_full_katakana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self.romkan.to_hiragana(self.preedit.as_str());
        let kata = hira2kata(hira.as_str(), ConvOption::default());
        self.convert_to_single(engine, hira.as_str(), kata.as_str())
    }

    /// convert to half-width katakana (standard katakana): ほわいと → ﾎﾜｲﾄ
    pub fn convert_to_half_katakana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self.romkan.to_hiragana(self.preedit.as_str());
        let kata = z2h(
            hira2kata(hira.as_str(), ConvOption::default()).as_str(),
            ConvOption::default(),
        );
        self.convert_to_single(engine, hira.as_str(), kata.as_str())
    }

    /// convert to full-width romaji, all-capitals, proper noun capitalization (latin script inside
    /// Japanese text): ホワイト → ｈｏｗａｉｔｏ → ＨＯＷＡＩＴＯ → Ｈｏｗａｉｔｏ
    pub fn convert_to_full_romaji(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self.romkan.to_hiragana(self.preedit.as_str());
        let romaji = h2z(
            &self.preedit,
            ConvOption {
                kana: true,
                digit: true,
                ascii: true,
                ..Default::default()
            },
        );
        self.convert_to_single(engine, hira.as_str(), romaji.as_str())
    }

    /// convert to half-width romaji, all-capitals, proper noun capitalization (latin script like
    /// standard English): ホワイト → howaito → HOWAITO → Howaito
    pub fn convert_to_half_romaji(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self.romkan.to_hiragana(self.preedit.as_str());
        let romaji = z2h(
            &self.preedit,
            ConvOption {
                kana: true,
                digit: true,
                ascii: true,
                ..Default::default()
            },
        );
        self.convert_to_single(engine, hira.as_str(), romaji.as_str())
    }

    /// 特定の1文節の文章を候補として表示する。
    /// F6 などを押した時用。
    fn convert_to_single(
        &mut self,
        engine: *mut IBusEngine,
        yomi: &str,
        surface: &str,
    ) -> Result<()> {
        // 候補を設定
        let candidate = Candidate::new(yomi, surface, 0_f32);
        let clauses = vec![VecDeque::from([candidate])];
        self.clauses = clauses;
        self.current_clause = 0;
        self.node_selected.clear();
        self.force_selected_clause = vec![];

        // ルックアップテーブルに候補を設定
        self.lookup_table.clear();
        let candidate = surface.to_ibus_text();
        self.lookup_table.append_candidate(candidate);

        // 表示を更新
        self.refresh(engine);
        Ok(())
    }

    pub fn escape(&mut self, engine: *mut IBusEngine) {
        trace!("escape: {}", self.preedit);
        self.preedit.clear();
        self.update_candidates(engine)
    }

    pub fn page_up(&mut self, engine: *mut IBusEngine) -> bool {
        if self.lookup_table.page_up() {
            self.node_selected.insert(
                self.current_clause,
                self.lookup_table.get_cursor_pos() as usize,
            );
            self.cursor_moved = true;
            self.refresh(engine);
            true
        } else {
            false
        }
    }

    pub fn page_down(&mut self, engine: *mut IBusEngine) -> bool {
        if self.lookup_table.page_up() {
            self.node_selected.insert(
                self.current_clause,
                self.lookup_table.get_cursor_pos() as usize,
            );
            self.cursor_moved = true;
            self.refresh(engine);
            true
        } else {
            false
        }
    }
}
