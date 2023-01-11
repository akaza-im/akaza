use crate::core::IBusSerializable;
use crate::glib::{g_object_ref_sink, gpointer, GArray};
use crate::property::IBusProperty;

extern "C" {
    pub fn ibus_prop_list_new() -> *mut IBusPropList;
    pub fn ibus_prop_list_append(prop_list: *mut IBusPropList, prop: *mut IBusProperty);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBusPropList {
    parent: IBusSerializable,
    candidates: *mut GArray,
}

impl Default for IBusPropList {
    fn default() -> Self {
        unsafe {
            let prop_list =
                g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;
            *prop_list
        }
    }
}

impl IBusPropList {
    pub fn append(&mut self, prop: *mut IBusProperty) {
        unsafe { ibus_prop_list_append(self as *mut _, prop as *mut _) }
    }
}
