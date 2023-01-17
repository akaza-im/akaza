use crate::core::IBusSerializable;
use crate::glib::{gboolean, gchar, gpointer};
use crate::prop_list::IBusPropList;
use crate::text::IBusText;

pub const IBusPropState_PROP_STATE_UNCHECKED: IBusPropState = 0;
pub const IBusPropState_PROP_STATE_CHECKED: IBusPropState = 1;
pub const IBusPropState_PROP_STATE_INCONSISTENT: IBusPropState = 2;

pub type IBusPropState = ::std::os::raw::c_uint;

pub const IBusPropType_PROP_TYPE_NORMAL: IBusPropType = 0;
pub const IBusPropType_PROP_TYPE_TOGGLE: IBusPropType = 1;
pub const IBusPropType_PROP_TYPE_RADIO: IBusPropType = 2;
pub const IBusPropType_PROP_TYPE_MENU: IBusPropType = 3;
pub const IBusPropType_PROP_TYPE_SEPARATOR: IBusPropType = 4;

pub type IBusPropType = ::std::os::raw::c_uint;

extern "C" {
    pub fn ibus_property_new(
        key: *const gchar,
        type_: IBusPropType,
        label: *mut IBusText,
        icon: *const gchar,
        tooltip: *mut IBusText,
        sensitive: gboolean,
        visible: gboolean,
        state: IBusPropState,
        prop_list: *mut IBusPropList,
    ) -> *mut IBusProperty;

    pub fn ibus_property_set_sub_props(prop: *mut IBusProperty, prop_list: *mut IBusPropList);

    pub fn ibus_property_set_state(prop: *mut IBusProperty, state: IBusPropState);

    pub fn ibus_property_set_label(prop: *mut IBusProperty, label: *mut IBusText);

    pub fn ibus_property_set_symbol(prop: *mut IBusProperty, symbol: *mut IBusText);

    pub fn ibus_property_set_icon(prop: *mut IBusProperty, icon: *const gchar);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBusPropertyPrivate {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBusProperty {
    parent: IBusSerializable,
    priv_: *mut IBusPropertyPrivate,
    pdummy: [gpointer; 7usize],
}
