use crate::glib::{gchar, guint};

extern "C" {
    pub fn ibus_keyval_name(keyval: guint) -> *const gchar;
    pub fn ibus_keyval_from_name(keyval_name: *const gchar) -> guint;
}
