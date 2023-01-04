use std::collections::HashMap;
use std::ffi::CString;

use log::{error, info, warn};

use ibus_sys::bindings::{
    gchar, ibus_engine_commit_text, ibus_engine_hide_preedit_text, ibus_lookup_table_clear,
    ibus_lookup_table_get_number_of_candidates, ibus_lookup_table_new, ibus_text_new_from_string,
    IBusEngine, IBusLookupTable,
};
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
}

impl Default for AkazaContext {
    fn default() -> Self {
        unsafe {
            AkazaContext {
                input_mode: InputMode::Hiragana,
                cursor_pos: 0,
                preedit: String::new(),
                //         self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
                lookup_table: ibus_lookup_table_new(10, 0, 1, 1),
                romkan: RomKanConverter::default(), // TODO make it configurable.
                command_map: ibus_akaza_commands_map(),
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
    /*
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
    */

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
        // TODO build string from clauses.
        self.preedit.clone()

        /*
        def build_string(self):
            result = ''
        for clauseid, nodes in enumerate(self.clauses):
            result += nodes[self.node_selected.get(clauseid, 0)].surface(lisp_evaluator)
        return result
        */
    }
}
