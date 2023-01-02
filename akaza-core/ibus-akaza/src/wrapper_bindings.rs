#![allow(non_camel_case_types)]

use crate::bindings::{gboolean, guint, IBusEngine};

// ibus-akaza original function in wrapper.c
pub type ibus_akaza_callback_key_event = unsafe extern "C" fn(
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> gboolean;

extern "C" {
    /// is_ibus: true if the project run with `--ibus` option.
    pub fn ibus_akaza_init(is_ibus: bool);

    pub fn ibus_akaza_set_callback(cb: ibus_akaza_callback_key_event);
}
