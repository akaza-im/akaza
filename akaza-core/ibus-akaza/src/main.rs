#![allow(non_upper_case_globals)]

use std::ffi::{c_void, CString};

use anyhow::Result;
use log::{info, warn};

use ibus_sys::bindings::{
    gboolean, gchar, guint, ibus_attr_list_append, ibus_attr_list_new, ibus_attribute_new,
    ibus_engine_commit_text, ibus_engine_hide_auxiliary_text, ibus_engine_hide_lookup_table,
    ibus_engine_hide_preedit_text, ibus_engine_update_preedit_text, ibus_lookup_table_clear,
    ibus_lookup_table_get_number_of_candidates, ibus_lookup_table_new, ibus_main,
    ibus_text_new_from_string, ibus_text_set_attributes, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, IBusEngine, IBusLookupTable,
    IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_MOD1_MASK,
    IBusModifierType_IBUS_RELEASE_MASK,
};
use ibus_sys::ibus_key::{IBUS_KEY_KP_Enter, IBUS_KEY_Return};
use libakaza::romkan::RomKanConverter;

use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod wrapper_bindings;

#[derive(Debug)]
enum KeyState {
    // 何も入力されていない状態。
    PreComposition,
    // 変換処理に入る前。ひらがなを入力している段階。
    Composition,
    // 変換中
    Conversion,
}

unsafe extern "C" fn process_key_event(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool {
    info!("process_key_event~~ {}, {}, {}", keyval, keycode, modifiers);

    // ignore key release event
    if modifiers & IBusModifierType_IBUS_RELEASE_MASK != 0 {
        return false;
    }
    let context_ref = &mut *(context as *mut AkazaContext);

    // keymap.register([KEY_STATE_COMPOSITION], ['Return', 'KP_Enter'], 'commit_preedit')
    let key_state = context_ref.get_key_state();

    // TODO configurable.
    info!("KeyState={:?}", key_state);
    match key_state {
        KeyState::PreComposition => {}
        KeyState::Composition => {
            match keyval {
                IBUS_KEY_Return | IBUS_KEY_KP_Enter => {
                    info!("commit_preedit");
                    context_ref
                        .commands
                        .commit_preedit(&mut *(context as *mut AkazaContext), engine);
                    return true;
                }
                IBUS_KEY_Backspace => {
                    context_ref.commands.erase_character_before_cursor(
                        &mut *(context as *mut AkazaContext),
                        engine,
                    );
                    return true;
                }
                _ => { /* do nothing. fallback to default process. */ }
            }
        }
        KeyState::Conversion => {
            match keyval {
                IBUS_KEY_Backspace => {
                    context_ref.commands.erase_character_before_cursor(
                        &mut *(context as *mut AkazaContext),
                        engine,
                    );
                    return true;
                }
                _ => { /* do nothing. fallback to default process. */ }
            }
        }
    }

    /*
        # 入力モードの切り替え
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['Henkan'],
                    'set_input_mode_hiragana')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-J'],
                    'set_input_mode_hiragana')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['Muhenkan'],
                    'set_input_mode_alnum')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-:'],
                    'set_input_mode_alnum')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-L'],
                    'set_input_mode_fullwidth_alnum')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-K'],
                    'set_input_mode_katakana')

    # 後から文字タイプを指定する
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F6'], 'convert_to_full_hiragana')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F7'], 'convert_to_full_katakana')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F8'], 'convert_to_half_katakana')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F9'], 'convert_to_full_romaji')
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F10'], 'convert_to_half_romaji')

    keymap.register([KEY_STATE_CONVERSION], ['space'], 'cursor_down')
    keymap.register([KEY_STATE_COMPOSITION], ['space'], 'update_candidates')

    keymap.register([KEY_STATE_CONVERSION], ['Return', 'KP_Enter'], 'commit_candidate')

    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['Escape'], 'escape')


    for n in range(0, 10):
        keymap.register([KEY_STATE_CONVERSION], [str(n), f"KP_{n}"], f"press_number_{n}")

    keymap.register([KEY_STATE_CONVERSION], ['Page_Up', 'KP_Page_Up'], 'page_up')
    keymap.register([KEY_STATE_CONVERSION], ['Page_Down', 'KP_Page_Down'], 'page_down')

    keymap.register([KEY_STATE_CONVERSION], ['Up', 'KP_Up'], 'cursor_up')
    keymap.register([KEY_STATE_CONVERSION], ['Down', 'KP_Down'], 'cursor_down')

    keymap.register([KEY_STATE_CONVERSION], ['Right', 'KP_Right'], 'cursor_right')
    keymap.register([KEY_STATE_CONVERSION], ['S-Right', 'S-KP_Right'], 'extend_clause_right')

    keymap.register([KEY_STATE_CONVERSION], ['Left', 'KP_Left'], 'cursor_left')
    keymap.register([KEY_STATE_CONVERSION], ['S-Left', 'S-KP_Left'], 'extend_clause_left')
     */

    match &context_ref.input_mode {
        InputMode::Hiragana => {
            if modifiers & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                != 0
            {
                return false;
            }

            if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                if ibus_lookup_table_get_number_of_candidates(context_ref.lookup_table) > 0 {
                    // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                    // TODO commit_candidate();
                }

                // Append the character to preedit string.
                let preedit = &mut context_ref.preedit;
                context_ref.preedit.push(char::from_u32(keyval).unwrap());
                context_ref.cursor_pos += 1;

                // And update the display status.
                update_preedit_text_before_henkan(context_ref, engine);
                return true;
            }
        }
        InputMode::Alnum => return false,
        _ => return false,
    }
    false // not proceeded by rust code.

    /*
        if ('!' <= keyval && keyval <= '~') {
      g_string_insert_c(akaza->preedit, akaza->cursor_pos, keyval);

      akaza->cursor_pos++;
      ibus_akaza_engine_update(akaza);

      return TRUE;
    }

       */
}

unsafe fn _make_preedit_word(context: &mut AkazaContext) -> (String, String) {
    let preedit = &context.preedit;
    // If the first character is upper case, return preedit string itself.
    if !preedit.is_empty() && preedit.chars().next().unwrap().is_ascii_uppercase() {
        // TODO: meaningless clone process.
        return (preedit.clone(), preedit.clone());
    }

    // TODO cache RomKanConverter instance
    let yomi = RomKanConverter::new().to_hiragana(preedit.as_str());
    (yomi.clone(), yomi)

    /*
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
    */
}

unsafe fn update_preedit_text_before_henkan(context: &mut AkazaContext, engine: *mut IBusEngine) {
    info!("update_preedit_text_before_henkan");
    if context.preedit.is_empty() {
        ibus_engine_hide_lookup_table(engine);
        return;
    }

    // Convert to Hiragana.
    info!("Convert to Hiragana");
    let (_yomi, word) = _make_preedit_word(context);

    let preedit_attrs = ibus_attr_list_new();
    ibus_attr_list_append(
        preedit_attrs,
        ibus_attribute_new(
            IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
            IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
            0,
            word.len() as guint,
        ),
    );
    let word_c_str = CString::new(word.clone()).unwrap();
    info!("Calling ibus_text_new_from_string");
    let preedit_text = ibus_text_new_from_string(word_c_str.as_ptr() as *const gchar);
    ibus_text_set_attributes(preedit_text, preedit_attrs);
    info!("Callihng ibus_engine_update_preedit_text");
    ibus_engine_update_preedit_text(
        engine,
        preedit_text,
        word.len() as guint,
        !word.is_empty() as gboolean,
    )

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

#[repr(C)]
enum InputMode {
    Hiragana,
    Alnum,
}

#[repr(C)]
#[derive(Default)]
struct Commands {}

impl Commands {
    fn commit_preedit(&self, context: &mut AkazaContext, engine: *mut IBusEngine) {
        /*
        # 無変換状態では、ひらがなに変換してコミットします。
        yomi, word = self._make_preedit_word()
        self.commit_string(word)
         */
        unsafe {
            let (_, surface) = _make_preedit_word(context);
            context.commit_string(engine, surface.as_str());
        }
    }

    fn erase_character_before_cursor(&self, context: &mut AkazaContext, engine: *mut IBusEngine) {
        info!("erase_character_before_cursor: {}", context.preedit);
        unsafe {
            if context.in_henkan_mode() {
                // 変換中の場合、無変換モードにもどす。
                ibus_lookup_table_clear(context.lookup_table);
                ibus_engine_hide_auxiliary_text(engine);
                ibus_engine_hide_lookup_table(engine);
            } else {
                // サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
                context.preedit = context.romkan.remove_last_char(&context.preedit)
            }
            // 変換していないときのレンダリングをする。
            update_preedit_text_before_henkan(context, engine);
        }
    }

    /*
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
    */
}

#[repr(C)]
struct AkazaContext {
    input_mode: InputMode,
    cursor_pos: i32,
    preedit: String,
    lookup_table: *mut IBusLookupTable,
    // TODO: rename to lookup_table
    commands: Commands,
    romkan: RomKanConverter,
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
                commands: Commands::default(),
                romkan: RomKanConverter::default(), // TODO make it configurable.
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
    fn get_key_state(&self) -> KeyState {
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

    fn in_henkan_mode(&self) -> bool {
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

    fn commit_string(&mut self, engine: *mut IBusEngine, text: &str) {
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
}

fn main() -> Result<()> {
    env_logger::init();

    unsafe {
        let mut ac = AkazaContext::default();

        ibus_akaza_set_callback(&mut ac as *mut _ as *mut c_void, process_key_event);

        ibus_akaza_init(true);

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}
