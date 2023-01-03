#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::bindings::guint;
use crate::bindings::IBusAkazaEngine;

pub const InputMode_ALNUM: InputMode = 0;
pub const InputMode_HIRAGANA: InputMode = 1;
pub type InputMode = ::std::os::raw::c_uint;

// ibus-akaza original function in wrapper.c
pub type ibus_akaza_callback_key_event = unsafe extern "C" fn(
    engine: *mut IBusAkazaEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool;

extern "C" {
    /// is_ibus: true if the project run with `--ibus` option.
    pub fn ibus_akaza_init(is_ibus: bool);

    pub fn ibus_akaza_set_callback(cb: ibus_akaza_callback_key_event);
}
