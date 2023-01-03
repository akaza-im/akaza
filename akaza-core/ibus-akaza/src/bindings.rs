#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// See bindgen.sh's output to improvement this file.

// ibus wrapper functions.

use crate::wrapper_bindings::InputMode;

pub type gchar = ::std::os::raw::c_char;
pub type guint = ::std::os::raw::c_uint;
pub type gboolean = ::std::os::raw::c_int;
pub type gsize = ::std::os::raw::c_ulong;
pub type gssize = ::std::os::raw::c_long;
pub type gint = ::std::os::raw::c_int;

pub const IBusModifierType_IBUS_SHIFT_MASK: IBusModifierType = 1;
pub const IBusModifierType_IBUS_LOCK_MASK: IBusModifierType = 2;
pub const IBusModifierType_IBUS_CONTROL_MASK: IBusModifierType = 4;
pub const IBusModifierType_IBUS_MOD1_MASK: IBusModifierType = 8;
pub const IBusModifierType_IBUS_MOD2_MASK: IBusModifierType = 16;
pub const IBusModifierType_IBUS_MOD3_MASK: IBusModifierType = 32;
pub const IBusModifierType_IBUS_MOD4_MASK: IBusModifierType = 64;
pub const IBusModifierType_IBUS_MOD5_MASK: IBusModifierType = 128;
pub const IBusModifierType_IBUS_BUTTON1_MASK: IBusModifierType = 256;
pub const IBusModifierType_IBUS_BUTTON2_MASK: IBusModifierType = 512;
pub const IBusModifierType_IBUS_BUTTON3_MASK: IBusModifierType = 1024;
pub const IBusModifierType_IBUS_BUTTON4_MASK: IBusModifierType = 2048;
pub const IBusModifierType_IBUS_BUTTON5_MASK: IBusModifierType = 4096;
pub const IBusModifierType_IBUS_HANDLED_MASK: IBusModifierType = 16777216;
pub const IBusModifierType_IBUS_FORWARD_MASK: IBusModifierType = 33554432;
pub const IBusModifierType_IBUS_IGNORED_MASK: IBusModifierType = 33554432;
pub const IBusModifierType_IBUS_SUPER_MASK: IBusModifierType = 67108864;
pub const IBusModifierType_IBUS_HYPER_MASK: IBusModifierType = 134217728;
pub const IBusModifierType_IBUS_META_MASK: IBusModifierType = 268435456;
pub const IBusModifierType_IBUS_RELEASE_MASK: IBusModifierType = 1073741824;
pub const IBusModifierType_IBUS_MODIFIER_MASK: IBusModifierType = 1593843711;
#[doc = " IBusModifierType:\n @IBUS_SHIFT_MASK: Shift  is activated.\n @IBUS_LOCK_MASK: Cap Lock is locked.\n @IBUS_CONTROL_MASK: Control key is activated.\n @IBUS_MOD1_MASK: Modifier 1 (Usually Alt_L (0x40),  Alt_R (0x6c),  Meta_L (0xcd)) activated.\n @IBUS_MOD2_MASK: Modifier 2 (Usually Num_Lock (0x4d)) activated.\n @IBUS_MOD3_MASK: Modifier 3 activated.\n @IBUS_MOD4_MASK: Modifier 4 (Usually Super_L (0xce),  Hyper_L (0xcf)) activated.\n @IBUS_MOD5_MASK: Modifier 5 (ISO_Level3_Shift (0x5c),  Mode_switch (0xcb)) activated.\n @IBUS_BUTTON1_MASK: Mouse button 1 (left) is activated.\n @IBUS_BUTTON2_MASK: Mouse button 2 (middle) is activated.\n @IBUS_BUTTON3_MASK: Mouse button 3 (right) is activated.\n @IBUS_BUTTON4_MASK: Mouse button 4 (scroll up) is activated.\n @IBUS_BUTTON5_MASK: Mouse button 5 (scroll down) is activated.\n @IBUS_HANDLED_MASK: Handled mask indicates the event has been handled by ibus.\n @IBUS_FORWARD_MASK: Forward mask indicates the event has been forward from ibus.\n @IBUS_IGNORED_MASK: It is an alias of IBUS_FORWARD_MASK.\n @IBUS_SUPER_MASK: Super (Usually Win) key is activated.\n @IBUS_HYPER_MASK: Hyper key is activated.\n @IBUS_META_MASK: Meta key is activated.\n @IBUS_RELEASE_MASK: Key is released.\n @IBUS_MODIFIER_MASK: Modifier mask for the all the masks above.\n\n Handles key modifier such as control, shift and alt and release event.\n Note that nits 15 - 25 are currently unused, while bit 29 is used internally."]
pub type IBusModifierType = ::std::os::raw::c_uint;

pub type IBusBus = [u64; 6usize];
pub type IBusText = [u64; 9usize];
pub type IBusLookupTable = [u64; 11usize];
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBusAkazaEngine {
    pub parent: IBusEngine,
    pub preedit: *mut GString,
    pub cursor_pos: gint,
    pub table: *mut IBusLookupTable,
    pub input_mode: InputMode,
}
pub type IBusEngine = [u64; 11usize];

extern "C" {
    pub fn g_string_insert_c(string: *mut GString, pos: gssize, c: gchar) -> *mut GString;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GString {
    pub str_: *mut gchar,
    pub len: gsize,
    pub allocated_len: gsize,
}

impl GString {
    /*
       g_string_insert_c(akaza->preedit, akaza->cursor_pos, keyval);
    */
    pub fn insert_c(&mut self, pos: gssize, c: gchar) {
        unsafe {
            g_string_insert_c(self, pos, c);
        }
    }
}

extern "C" {
    pub fn ibus_bus_new() -> *mut IBusBus;
    pub fn ibus_init();
    pub fn ibus_main();

    // text
    pub fn ibus_text_new_from_string(str_: *const gchar) -> *mut IBusText;

    // lookup
    pub fn ibus_lookup_table_clear(table: *mut IBusLookupTable);
    #[doc = " ibus_lookup_table_get_number_of_candidates:\n @table: An IBusLookupTable.\n\n Return the number of candidate in the table.\n\n Returns: The number of candidates in the table"]
    pub fn ibus_lookup_table_get_number_of_candidates(table: *mut IBusLookupTable) -> guint;

    // engine
    pub fn ibus_engine_commit_text(engine: *mut IBusEngine, text: *mut IBusText);
    pub fn ibus_engine_hide_lookup_table(engine: *mut IBusAkazaEngine);
}
