#![allow(non_upper_case_globals)]

use std::ffi::{c_void, CString};

use anyhow::Result;
use log::{info, warn};

use libakaza::romkan::RomKanConverter;

use crate::bindings::{
    gboolean, gchar, gssize, guint, ibus_attr_list_append, ibus_attr_list_new, ibus_attribute_new,
    ibus_engine_hide_lookup_table, ibus_engine_update_preedit_text, ibus_lookup_table_clear,
    ibus_lookup_table_get_number_of_candidates, ibus_lookup_table_new, ibus_main,
    ibus_text_new_from_string, ibus_text_set_attributes, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, IBusEngine, IBusLookupTable,
    IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_MOD1_MASK,
    IBusModifierType_IBUS_RELEASE_MASK,
};
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod bindings;
mod wrapper_bindings;

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
    // let mut engine = *engine;

    // ignore key release event
    if modifiers & IBusModifierType_IBUS_RELEASE_MASK != 0 {
        return false;
    }
    let context = &mut *(context as *mut AkazaContext);

    match &context.input_mode {
        InputMode::Hiragana => {
            if modifiers & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                != 0
            {
                return false;
            }

            if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                if ibus_lookup_table_get_number_of_candidates(context.table) > 0 {
                    // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                    // TODO commit_candidate();
                }

                // Append the character to preedit string.
                let preedit = &mut context.preedit;
                context.preedit.push(char::from_u32(keyval).unwrap());
                context.cursor_pos += 1;

                // And update the display status.
                update_preedit_text_before_henkan(context, engine);
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

unsafe fn _make_preedit_word(
    context: &mut AkazaContext,
    engine: *mut IBusEngine,
) -> (String, String) {
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
    let (_yomi, word) = _make_preedit_word(context, engine);

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
struct AkazaContext {
    input_mode: InputMode,
    cursor_pos: i32,
    preedit: String,
    table: *mut IBusLookupTable,
}

impl Default for AkazaContext {
    fn default() -> Self {
        unsafe {
            AkazaContext {
                input_mode: InputMode::Hiragana,
                cursor_pos: 0,
                preedit: String::new(),
                //         self.lookup_table = IBus.LookupTable.new(page_size=10, cursor_pos=0, cursor_visible=True, round=True)
                table: ibus_lookup_table_new(10, 0, 1, 1),
            }
        }
    }
}

impl Drop for AkazaContext {
    fn drop(&mut self) {
        warn!("Dropping AkazaContext");
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
