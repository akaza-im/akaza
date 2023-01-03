#![allow(non_upper_case_globals)]
use anyhow::Result;
use flexi_logger::{FileSpec, Logger};
use log::info;

use crate::bindings::{
    gchar, gssize, guint, ibus_engine_hide_lookup_table, ibus_lookup_table_clear,
    ibus_lookup_table_get_number_of_candidates, ibus_main, IBusAkazaEngine,
    IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_MOD1_MASK,
    IBusModifierType_IBUS_RELEASE_MASK,
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

            if ibus_lookup_table_get_number_of_candidates((*engine).table) > 0 {
                // TODO commit_candidate();
            }

            if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                let preedit = (*engine).preedit;
                (*preedit).insert_c((*engine).cursor_pos as gssize, keyval as gchar);
                (*engine).cursor_pos += 1;
                ibus_akaza_engine_update(engine);
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
        .log_to_file(
            FileSpec::default()
                .directory("/tmp") // create files in folder ./log_files
                .basename("ibus-akaza")
                .discriminant("Sample4711A") // use infix in log file name
                .suffix("trc"), // use suffix .trc instead of .log
        )
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
