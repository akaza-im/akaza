#![allow(non_camel_case_types)]
#![allow(dead_code)]

// See bindgen.sh's output to improvement this file.

// ibus wrapper functions.
pub type IBusBus = [u64; 6usize];
pub type IBusText = [u64; 9usize];
pub type IBusEngine = [u64; 11usize];

pub type gchar = ::std::os::raw::c_char;
pub type guint = ::std::os::raw::c_uint;
pub type gboolean = ::std::os::raw::c_int;
pub type gsize = ::std::os::raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GString {
    pub str_: *mut gchar,
    pub len: gsize,
    pub allocated_len: gsize,
}

extern "C" {
    pub fn ibus_bus_new() -> *mut IBusBus;
    pub fn ibus_init();
    pub fn ibus_main();

    // text
    pub fn ibus_text_new_from_string(str_: *const gchar) -> *mut IBusText;

    // engine
    pub fn ibus_engine_commit_text(engine: *mut IBusEngine, text: *mut IBusText);
}
