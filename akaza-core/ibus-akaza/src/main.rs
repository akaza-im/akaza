#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::time::SystemTime;

use anyhow::Result;
use log::{info, warn};

use ibus_sys::bindings::guint;
use ibus_sys::bindings::ibus_main;
use ibus_sys::bindings::IBusEngine;
use ibus_sys::bindings::IBusModifierType_IBUS_CONTROL_MASK;
use ibus_sys::bindings::IBusModifierType_IBUS_MOD1_MASK;
use ibus_sys::bindings::IBusModifierType_IBUS_RELEASE_MASK;
use ibus_sys::lookup_table::ibus_lookup_table_get_number_of_candidates;
use libakaza::akaza_builder::AkazaBuilder;

use crate::context::AkazaContext;
use crate::keymap::KeyMap;
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod commands;
mod context;
mod keymap;
mod wrapper_bindings;

#[derive(Debug, Hash, PartialEq, Copy, Clone)]
pub enum KeyState {
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
    info!(
        "process_key_event: keyval={}, keycode={}, modifiers={}",
        keyval, keycode, modifiers
    );

    // ignore key release event
    if modifiers & IBusModifierType_IBUS_RELEASE_MASK != 0 {
        return false;
    }
    let context_ref = &mut *(context as *mut AkazaContext);

    // keymap.register([KEY_STATE_COMPOSITION], ['Return', 'KP_Enter'], 'commit_preedit')
    let key_state = context_ref.get_key_state();

    // TODO configurable.
    info!("KeyState={:?}", key_state);
    let keymap = KeyMap::new();

    if let Some(cb) = keymap.get(&key_state, keyval) {
        return context_ref.run_callback_by_name(engine, cb);
    }

    match &context_ref.input_mode {
        InputMode::Hiragana => {
            if modifiers & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                != 0
            {
                return false;
            }

            if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                info!("Insert new character to preedit: '{}'", context_ref.preedit);
                if ibus_lookup_table_get_number_of_candidates(context_ref.lookup_table) > 0 {
                    // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                    // TODO commit_candidate();
                }

                // Append the character to preedit string.
                context_ref.preedit.push(char::from_u32(keyval).unwrap());
                context_ref.cursor_pos += 1;

                // And update the display status.
                context_ref.update_preedit_text_before_henkan(engine);
                return true;
            }
        }
        InputMode::Alnum => return false,
        // _ => return false,
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

    let yomi = context.romkan.to_hiragana(preedit.as_str());
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

#[repr(C)]
#[derive(Debug)]
enum InputMode {
    Hiragana,
    Alnum,
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Starting ibus-akaza(rust version)");

    unsafe {
        let sys_time = SystemTime::now();
        let akaza = AkazaBuilder::default()
            // TODO take dictionary path from command line option.
            .system_data_dir("/home/tokuhirom/dev/akaza/akaza-data/data")
            .build()?;
        let mut ac = AkazaContext::new(akaza);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(sys_time)?;
        info!(
            "Initialized ibus-akaza engine in {} milliseconds.",
            difference.as_millis()
        );

        ibus_akaza_set_callback(&mut ac as *mut _ as *mut c_void, process_key_event);

        ibus_akaza_init(true);

        info!("Enter the ibus_main()");

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
