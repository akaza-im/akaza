#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::ffi::c_void;

use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::guint;

// FFI for the wrapper.c
pub(crate) type ibus_akaza_callback_key_event = unsafe extern "C" fn(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool;

pub(crate) type ibus_akaza_callback_candidate_clicked = unsafe extern "C" fn(
    context: *mut c_void,
    engine: *mut IBusEngine,
    index: guint,
    button: guint,
    state: guint,
);

pub(crate) type ibus_akaza_callback_focus_in =
    unsafe extern "C" fn(context: *mut c_void, engine: *mut IBusEngine);

extern "C" {
    /// is_ibus: true if the project run with `--ibus` option.
    pub fn ibus_akaza_init(is_ibus: bool);

    pub(crate) fn ibus_akaza_set_callback(
        context: *mut c_void,
        key_event_cb: ibus_akaza_callback_key_event,
        candidate_cb: ibus_akaza_callback_candidate_clicked,
        focus_in_cb: ibus_akaza_callback_focus_in,
    );
}
