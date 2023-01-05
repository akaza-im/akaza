#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::time::SystemTime;

use anyhow::Result;
use log::{info, trace, warn};

use ibus_sys::core::ibus_main;
use ibus_sys::core::IBusModifierType_IBUS_CONTROL_MASK;
use ibus_sys::core::IBusModifierType_IBUS_MOD1_MASK;
use ibus_sys::core::IBusModifierType_IBUS_RELEASE_MASK;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::guint;
use libakaza::akaza_builder::AkazaBuilder;

use crate::context::{AkazaContext, InputMode};
use crate::keymap::KeyMap;
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod commands;
mod context;
mod keymap;
mod wrapper_bindings;

unsafe extern "C" fn process_key_event(
    context: *mut c_void,
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
    let context_ref = &mut *(context as *mut AkazaContext);

    // keymap.register([KEY_STATE_COMPOSITION], ['Return', 'KP_Enter'], 'commit_preedit')
    let key_state = context_ref.get_key_state();

    // TODO configure keymap in ~/.config/akaza/keymap.yml?
    trace!("KeyState={:?}", key_state);
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
                if context_ref.lookup_table.get_number_of_candidates() > 0 {
                    // 変換の途中に別の文字が入力された。よって、現在の preedit 文字列は確定させる。
                    context_ref.commit_candidate(engine);
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
