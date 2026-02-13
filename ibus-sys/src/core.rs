#![allow(dead_code)]

// See bindgen.sh's output to improvement this file.
// TODO: maybe i can update this file as more rust native interface...

// ibus wrapper functions.

use crate::glib::gboolean;

pub type IBusSerializable = [u64; 6usize];

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

extern "C" {
    pub fn ibus_bus_new() -> *mut IBusBus;
    pub fn ibus_init();
    pub fn ibus_main();

}

pub fn to_gboolean(b: bool) -> gboolean {
    i32::from(b)
}
