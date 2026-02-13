use crate::core::IBusSerializable;
use crate::glib::GArray;
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
