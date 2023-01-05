extern "C" {
    // This method retain the object's reference count.n
    pub fn g_object_ref_sink(object: gpointer) -> gpointer;
}

pub type gchar = ::std::os::raw::c_char;
pub type guint = ::std::os::raw::c_uint;
pub type gboolean = ::std::os::raw::c_int;
pub type gsize = ::std::os::raw::c_ulong;
pub type gssize = ::std::os::raw::c_long;
pub type gint = ::std::os::raw::c_int;
pub type gpointer = *mut ::std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GArray {
    pub data: *mut gchar,
    pub len: guint,
}
