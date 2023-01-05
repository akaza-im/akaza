#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// See bindgen.sh's output to improvement this file.
// TODO: maybe i can update this file as more rust native interface...

// ibus wrapper functions.

use crate::lookup_table::IBusLookupTable;
use std::ffi::CString;

pub type gchar = ::std::os::raw::c_char;
pub type guint = ::std::os::raw::c_uint;
pub type gboolean = ::std::os::raw::c_int;
pub type gsize = ::std::os::raw::c_ulong;
pub type gssize = ::std::os::raw::c_long;
pub type gint = ::std::os::raw::c_int;
pub type gpointer = *mut ::std::os::raw::c_void;

pub type GArray = _GArray;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _GArray {
    pub data: *mut gchar,
    pub len: guint,
}

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

pub const IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE: IBusAttrType = 1;
pub const IBusAttrType_IBUS_ATTR_TYPE_FOREGROUND: IBusAttrType = 2;
pub const IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND: IBusAttrType = 3;

#[doc = " IBusAttrType:\n @IBUS_ATTR_TYPE_UNDERLINE: Decorate with underline.\n @IBUS_ATTR_TYPE_FOREGROUND: Foreground color.\n @IBUS_ATTR_TYPE_BACKGROUND: Background color.\n\n Type enumeration of IBusText attribute."]
pub type IBusAttrType = ::std::os::raw::c_uint;

pub const IBusAttrUnderline_IBUS_ATTR_UNDERLINE_NONE: IBusAttrUnderline = 0;
pub const IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE: IBusAttrUnderline = 1;
pub const IBusAttrUnderline_IBUS_ATTR_UNDERLINE_DOUBLE: IBusAttrUnderline = 2;
pub const IBusAttrUnderline_IBUS_ATTR_UNDERLINE_LOW: IBusAttrUnderline = 3;
pub const IBusAttrUnderline_IBUS_ATTR_UNDERLINE_ERROR: IBusAttrUnderline = 4;

#[doc = " IBusAttrUnderline:\n @IBUS_ATTR_UNDERLINE_NONE: No underline.\n @IBUS_ATTR_UNDERLINE_SINGLE: Single underline.\n @IBUS_ATTR_UNDERLINE_DOUBLE: Double underline.\n @IBUS_ATTR_UNDERLINE_LOW: Low underline ? FIXME\n @IBUS_ATTR_UNDERLINE_ERROR: Error underline\n\n Type of IBusText attribute."]
pub type IBusAttrUnderline = ::std::os::raw::c_uint;

pub type IBusBus = [u64; 6usize];
pub type IBusText = [u64; 9usize];

pub type IBusAttrList = [u64; 7usize];
#[doc = " IBusAttribute:\n @type: IBusAttributeType\n @value: Value for the type.\n @start_index: The starting index, inclusive.\n @end_index: The ending index, exclusive.\n\n Signify the type, value and scope of the attribute.\n The scope starts from @start_index till the @end_index-1."]
pub type IBusAttribute = [u64; 8usize];

pub type IBusEngine = [u64; 11usize];

extern "C" {
    pub fn ibus_bus_new() -> *mut IBusBus;
    pub fn ibus_init();
    pub fn ibus_main();

    // attr
    #[doc = " ibus_attr_list_new:\n\n Creates an new #IBusAttrList.\n\n Returns: A newly allocated #IBusAttrList."]
    pub fn ibus_attr_list_new() -> *mut IBusAttrList;
    #[doc = " ibus_attr_list_append:\n @attr_list: An IBusAttrList instance.\n @attr: The IBusAttribute instance to be appended.\n\n Append an IBusAttribute to IBusAttrList, and increase reference."]
    pub fn ibus_attr_list_append(attr_list: *mut IBusAttrList, attr: *mut IBusAttribute);

    // attribute
    #[doc = " ibus_attribute_new:\n @type: Type of the attribute.\n @value: Value of the attribute.\n @start_index: Where attribute starts.\n @end_index: Where attribute ends.\n\n Creates a new IBusAttribute.\n\n Returns: (transfer none): A newly allocated IBusAttribute."]
    pub fn ibus_attribute_new(
        type_: guint,
        value: guint,
        start_index: guint,
        end_index: guint,
    ) -> *mut IBusAttribute;

    // text
    pub fn ibus_text_new_from_string(str_: *const gchar) -> *mut IBusText;
    #[doc = " ibus_text_set_attributes:\n @text: An IBusText.\n @attrs: An IBusAttrList"]
    pub fn ibus_text_set_attributes(text: *mut IBusText, attrs: *mut IBusAttrList);

    // engine
    pub fn ibus_engine_commit_text(engine: *mut IBusEngine, text: *mut IBusText);
    pub fn ibus_engine_hide_lookup_table(engine: *mut IBusEngine);
    #[doc = " ibus_engine_update_preedit_text:\n @engine: An IBusEngine.\n @text: Update content.\n @cursor_pos: Current position of cursor\n @visible: Whether the pre-edit buffer is visible.\n\n Update the pre-edit buffer.\n\n (Note: The text object will be released, if it is floating.\n  If caller want to keep the object, caller should make the object\n  sink by g_object_ref_sink.)"]
    pub fn ibus_engine_update_preedit_text(
        engine: *mut IBusEngine,
        text: *mut IBusText,
        cursor_pos: guint,
        visible: gboolean,
    );
    #[doc = " ibus_engine_hide_preedit_text:\n @engine: An IBusEngine.\n\n Hide the pre-edit buffer."]
    pub fn ibus_engine_hide_preedit_text(engine: *mut IBusEngine);
    #[doc = " ibus_engine_hide_auxiliary_text:\n @engine: An IBusEngine.\n\n Hide the auxiliary bar."]
    pub fn ibus_engine_hide_auxiliary_text(engine: *mut IBusEngine);
    #[doc = " ibus_engine_update_auxiliary_text:\n @engine: An IBusEngine.\n @text: Update content.\n @visible: Whether the auxiliary text bar is visible.\n\n Update the auxiliary bar.\n\n (Note: The text object will be released, if it is floating.\n  If caller want to keep the object, caller should make the object\n  sink by g_object_ref_sink.)"]
    pub fn ibus_engine_update_auxiliary_text(
        engine: *mut IBusEngine,
        text: *mut IBusText,
        visible: gboolean,
    );
    #[doc = " ibus_engine_update_lookup_table:\n @engine: An IBusEngine.\n @lookup_table: An lookup_table.\n @visible: Whether the lookup_table is visible.\n\n Update the lookup table.\n\n (Note: The table object will be released, if it is floating.\n  If caller want to keep the object, caller should make the object\n  sink by g_object_ref_sink.)"]
    pub fn ibus_engine_update_lookup_table(
        engine: *mut IBusEngine,
        lookup_table: *mut IBusLookupTable,
        visible: gboolean,
    );
}

pub trait StringExt {
    fn to_ibus_text(&self) -> *mut IBusText;
}

impl StringExt for str {
    fn to_ibus_text(&self) -> *mut IBusText {
        unsafe {
            let text_c_str = CString::new(self).unwrap();
            ibus_text_new_from_string(text_c_str.as_ptr() as *const gchar)
        }
    }
}

pub fn to_gboolean(b: bool) -> gboolean {
    i32::from(b)
}
