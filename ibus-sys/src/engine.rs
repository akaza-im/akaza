use crate::glib::{gboolean, guint};
use crate::lookup_table::IBusLookupTable;
use crate::prop_list::IBusPropList;
use crate::text::IBusText;

extern "C" {
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

    pub fn ibus_engine_register_properties(engine: *mut IBusEngine, prop_list: *mut IBusPropList);
}

pub type IBusEngine = [u64; 11usize];
