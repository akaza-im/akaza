#![allow(non_upper_case_globals)]

use anyhow::Result;
use flexi_logger::{FileSpec, Logger};
use libakaza::romkan::RomKanConverter;
use log::info;
use std::ffi::CString;

use crate::bindings::{
    gboolean, gchar, gssize, guint, ibus_attr_list_append, ibus_attr_list_new, ibus_attribute_new,
    ibus_engine_hide_lookup_table, ibus_engine_update_preedit_text, ibus_lookup_table_clear,
    ibus_lookup_table_get_number_of_candidates, ibus_main, ibus_text_new_from_string,
    ibus_text_set_attributes, IBusAkazaEngine, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, IBusModifierType_IBUS_CONTROL_MASK,
    IBusModifierType_IBUS_MOD1_MASK, IBusModifierType_IBUS_RELEASE_MASK,
};
use crate::wrapper_bindings::{
    ibus_akaza_init, ibus_akaza_set_callback, InputMode_ALNUM, InputMode_HIRAGANA,
};

mod bindings;
mod wrapper_bindings;

unsafe extern "C" fn process_key_event(
    engine: *mut IBusAkazaEngine,
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

    match (*engine).input_mode {
        InputMode_HIRAGANA => {
            if modifiers & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                != 0
            {
                return false;
            }

            if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                if ibus_lookup_table_get_number_of_candidates((*engine).table) > 0 {
                    // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                    // TODO commit_candidate();
                }

                // Append the character to preedit string.
                let preedit = (*engine).preedit;
                (*preedit).insert_c((*engine).cursor_pos as gssize, keyval as gchar);
                (*engine).cursor_pos += 1;

                // And update the display status.
                update_preedit_text_before_henkan(engine);
                return true;
            }
        }
        InputMode_ALNUM => return false,
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

unsafe fn _make_preedit_word(engine: *mut IBusAkazaEngine) -> (String, String) {
    let preedit = (*(*engine).preedit).as_string();
    // If the first character is upper case, return preedit string itself.
    if preedit.len() > 0 && preedit.chars().nth(0).unwrap().is_ascii_uppercase() {
        // TODO: meaningless clone process.
        return (preedit.clone(), preedit);
    }

    // TODO cache RomKanConverter instance
    let yomi = RomKanConverter::new().to_hiragana(preedit.as_str());
    return (yomi.clone(), yomi);

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

unsafe fn update_preedit_text_before_henkan(engine: *mut IBusAkazaEngine) {
    info!("update_preedit_text_before_henkan");
    if (*(*engine).preedit).len == 0 {
        ibus_engine_hide_lookup_table(engine);
        return;
    }

    // Convert to Hiragana.
    info!("Convert to Hiragana");
    let (_yomi, word) = _make_preedit_word(engine);

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

// TODO deprecate this?
unsafe fn ibus_akaza_engine_update(engine: *mut IBusAkazaEngine) {
    info!("ibus_akaza_engine_update: {}", (*(*engine).preedit).len);

    if (*(*engine).preedit).len == 0 {
        ibus_engine_hide_lookup_table(engine);
        return;
    }

    ibus_lookup_table_clear((*engine).table);

    // TODO ここで変換処理を行う。
    let sugs: Vec<String> = vec![];

    if sugs.is_empty() {
        // There's no candidates... is this possible?
        ibus_engine_hide_lookup_table(engine);
    }

    /*

    if (akaza->preedit->len == 0) {
      ibus_engine_hide_lookup_table((IBusEngine *)akaza);
      return;
    }

    ibus_lookup_table_clear(akaza->table);

    // XXX i need to implement kana-kanji conversion here.
    sugs = enchant_dict_suggest(dict, akaza->preedit->str,
                                akaza->preedit->len, &n_sug);

    if (sugs == NULL || n_sug == 0) {
      ibus_engine_hide_lookup_table((IBusEngine *)akaza);
      return;
    }

    for (i = 0; i < n_sug; i++) {
      ibus_lookup_table_append_candidate(akaza->table,
                                         ibus_text_new_from_string(sugs[i]));
    }

    ibus_engine_update_lookup_table((IBusEngine *)akaza, akaza->table, TRUE);

       */
}

fn main() -> Result<()> {
    Logger::try_with_str("info")?
        .print_message() //
        .start()?;

    unsafe {
        ibus_akaza_set_callback(process_key_event);

        ibus_akaza_init(true);

        // run main loop
        ibus_main();
    }
    Ok(())
}
