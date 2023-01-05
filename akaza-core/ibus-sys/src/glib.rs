use crate::core::gpointer;

extern "C" {
    // This method retain the object's reference count.n
    pub fn g_object_ref_sink(object: gpointer) -> gpointer;
}
