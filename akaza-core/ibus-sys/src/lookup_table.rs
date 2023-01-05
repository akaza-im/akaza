use crate::core::IBusSerializable;
use crate::glib::{g_object_ref_sink, gboolean, gint, gpointer, guint, GArray};
use crate::text::IBusText;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBusLookupTable {
    pub parent: IBusSerializable,
    pub page_size: guint,
    pub cursor_pos: guint,
    pub cursor_visible: gboolean,
    pub round: gboolean,
    pub orientation: gint,
    pub candidates: *mut GArray,
    pub labels: *mut GArray,
}

impl IBusLookupTable {
    pub fn new(
        page_size: guint,
        cursor_pos: guint,
        cursor_visible: gboolean,
        round: gboolean,
    ) -> Self {
        unsafe {
            let lookup_table = g_object_ref_sink(ibus_lookup_table_new(
                page_size,
                cursor_pos,
                cursor_visible,
                round,
            ) as gpointer) as *mut IBusLookupTable;
            *lookup_table
        }
    }

    pub fn get_number_of_candidates(&mut self) -> guint {
        // info!("get_number_of_candidates: {:?}", self);
        unsafe { ibus_lookup_table_get_number_of_candidates(self as *mut Self) }
    }

    pub fn clear(&mut self) {
        unsafe { ibus_lookup_table_clear(self as *mut Self) }
    }

    pub fn cursor_down(&mut self) -> bool {
        unsafe {
            let b = ibus_lookup_table_cursor_down(self as *mut _);
            b == 1
        }
    }
    pub fn get_cursor_pos(&mut self) -> guint {
        unsafe { ibus_lookup_table_get_cursor_pos(self as *mut _) }
    }
}

extern "C" {
    // lookup
    #[doc = " ibus_lookup_table_new:\n @page_size: number of candidate shown per page, the max value is 16.\n @cursor_pos: position index of cursor.\n @cursor_visible: whether the cursor is visible.\n @round: TRUE for lookup table wrap around.\n\n Craetes a new #IBusLookupTable.\n\n Returns: A newly allocated #IBusLookupTable."]
    pub fn ibus_lookup_table_new(
        page_size: guint,
        cursor_pos: guint,
        cursor_visible: gboolean,
        round: gboolean,
    ) -> *mut IBusLookupTable;
    pub fn ibus_lookup_table_clear(table: *mut IBusLookupTable);
    #[doc = " ibus_lookup_table_get_number_of_candidates:\n @table: An IBusLookupTable.\n\n Return the number of candidate in the table.\n\n Returns: The number of candidates in the table"]
    pub fn ibus_lookup_table_get_number_of_candidates(table: *mut IBusLookupTable) -> guint;
    #[doc = " ibus_lookup_table_append_candidate:\n @table: An IBusLookupTable.\n @text: candidate word/phrase to be appended (in IBusText format).\n\n Append a candidate word/phrase to IBusLookupTable, and increase reference."]
    pub fn ibus_lookup_table_append_candidate(table: *mut IBusLookupTable, text: *mut IBusText);
    pub fn ibus_lookup_table_cursor_down(table: *mut IBusLookupTable) -> gboolean;
    pub fn ibus_lookup_table_get_cursor_pos(table: *mut IBusLookupTable) -> guint;
}
