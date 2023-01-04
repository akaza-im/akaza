use std::collections::{HashMap, VecDeque};
use std::ffi::CString;

use anyhow::Result;
use log::{error, info, warn};

use ibus_sys::bindings::ibus_engine_update_auxiliary_text;
use ibus_sys::bindings::ibus_lookup_table_clear;
use ibus_sys::bindings::ibus_lookup_table_get_number_of_candidates;
use ibus_sys::bindings::ibus_lookup_table_new;
use ibus_sys::bindings::ibus_text_new_from_string;
use ibus_sys::bindings::IBusEngine;
use ibus_sys::bindings::IBusLookupTable;
use ibus_sys::bindings::{gchar, StringExt};
use ibus_sys::bindings::{
    gsize, guint, ibus_attr_list_append, ibus_attribute_new, ibus_engine_commit_text,
    IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
};
use ibus_sys::bindings::{
    ibus_attr_list_new, ibus_engine_hide_lookup_table, ibus_text_set_attributes,
};
use ibus_sys::bindings::{ibus_engine_hide_auxiliary_text, to_gboolean};
use ibus_sys::bindings::{ibus_engine_hide_preedit_text, ibus_engine_update_preedit_text};
use libakaza::akaza_builder::Akaza;
use libakaza::graph::graph_resolver::Candidate;
use libakaza::romkan::RomKanConverter;

use crate::commands::{ibus_akaza_commands_map, IbusAkazaCommand};
use crate::{InputMode, KeyState};

#[repr(C)]
pub struct AkazaContext {
    pub(crate) input_mode: InputMode,
    pub(crate) cursor_pos: i32,
    pub(crate) preedit: String,
    pub(crate) lookup_table: *mut IBusLookupTable,
    pub(crate) romkan: RomKanConverter,
    command_map: HashMap<&'static str, IbusAkazaCommand>,
    akaza: Akaza,
    clauses: Vec<VecDeque<Candidate>>,
    // げんざいせんたくされているぶんせつ。
    current_clause: usize,
}

impl AkazaContext {
    pub(crate) fn new(akaza: Akaza) -> Self {
        unsafe {
            AkazaContext {
                input_mode: InputMode::Hiragana,
                cursor_pos: 0,
                preedit: String::new(),
                //         self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
                lookup_table: ibus_lookup_table_new(10, 0, 1, 1),
                romkan: RomKanConverter::default(), // TODO make it configurable.
                command_map: ibus_akaza_commands_map(),
                akaza,
                clauses: vec![],
                current_clause: 0,
            }
        }
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
        /*
        def in_henkan_mode(self):
            return self.lookup_table.get_number_of_candidates() > 0
         */
        unsafe { ibus_lookup_table_get_number_of_candidates(self.lookup_table) > 0 }
    }

    pub fn commit_string(&mut self, engine: *mut IBusEngine, text: &str) {
        unsafe {
            let text_c_str = CString::new(text.clone()).unwrap();
            ibus_engine_commit_text(
                engine,
                ibus_text_new_from_string(text_c_str.as_ptr() as *const gchar),
            );
            self.preedit.clear();
            ibus_lookup_table_clear(self.lookup_table);
            ibus_engine_hide_preedit_text(engine);
        }

        /*
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
         */
    }

    fn commit_candidate(&mut self, engine: *mut IBusEngine) {
        let s = self.build_string();
        self.commit_string(engine, s.as_str());
        /*
        def commit_candidate(self):
            self.logger.info("commit_candidate")
            s = self.build_string()
            self.logger.info(f"Committing {s}")
            self.commit_string(s)
         */
    }

    fn build_string(&self) -> String {
        let mut result = String::new();
        for (clauseid, nodes) in self.clauses.iter().enumerate() {
            // TODO lisp をひょうかする
            // TODO node_selected をひょうかする
            // result += nodes[self.node_selected.get(clauseid, 0)].surface(lisp_evaluator)
            result += &nodes[clauseid].kanji;
        }
        result
    }

    pub(crate) fn update_candidates(&mut self, engine: *mut IBusEngine) {
        self._update_candidates(engine).unwrap();
        // TODO more processing

        /*
           def update_candidates(self):
           self.logger.info("update_candidates")
           try:
               self._update_candidates()
               self.current_clause = 0
               self.node_selected = {}
           except:
               self.logger.error(f"cannot get kanji candidates {sys.exc_info()[0]}", exc_info=True)
        */
    }

    fn _update_candidates(&mut self, engine: *mut IBusEngine) -> Result<()> {
        if self.preedit.is_empty() {
            self.clauses = vec![]
        } else {
            // TODO support force selected.
            self.clauses = self.akaza.convert(self.preedit.as_str(), &vec![])?;
        }
        self.create_lookup_table();
        self.refresh(engine);
        Ok(())
        /*
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
        */
    }

    /**
     * 現在の候補選択状態から、 lookup table を構築する。
     */
    fn create_lookup_table(&mut self) {
        unsafe {
            // 一旦、ルックアップテーブルをクリアする
            ibus_lookup_table_clear(self.lookup_table);

            // 現在の未変換情報を元に、候補を算出していく。
            if !self.clauses.is_empty() {
                // TODO implement here.
                /*
                   # lookup table に候補を詰め込んでいく。
                   for node in self.clauses[self.current_clause]:
                       candidate = IBus.Text.new_from_string(node.surface(lisp_evaluator))
                       self.lookup_table.append_candidate(candidate)
                */
            }
        }

        /*
        def create_lookup_table(self):
            # 一旦、ルックアップテーブルをクリアする
            self.lookup_table.clear()

            # 現在の未変換情報を元に、候補を産出していく。
            if len(self.clauses) > 0:
         */
    }

    fn refresh(&self, engine: *mut IBusEngine) {
        unsafe {
            // preedit_len = len(self.preedit_string)

            if self.clauses.is_empty() {
                ibus_engine_hide_auxiliary_text(engine);
                ibus_engine_hide_lookup_table(engine);
                ibus_engine_hide_preedit_text(engine);
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
            // bgstart = sum([len(self.clauses[i][0].surface(lisp_evaluator)) for i in 0..self.current_clause])
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

            // TODO following things.
            // # 候補があれば、選択肢を表示させる。
            // self._update_lookup_table()
            // self.is_invalidate = False
        }
    }
}
