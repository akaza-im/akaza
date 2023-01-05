#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use ibus_sys::bindings::guint;

use ibus_sys::engine::IBusEngine;
use std::ffi::c_void;

// ibus-akaza original function in wrapper.c
pub(crate) type ibus_akaza_callback_key_event = unsafe extern "C" fn(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool;

extern "C" {
    /// is_ibus: true if the project run with `--ibus` option.
    pub fn ibus_akaza_init(is_ibus: bool);

    pub(crate) fn ibus_akaza_set_callback(context: *mut c_void, cb: ibus_akaza_callback_key_event);
}
