use crate::attribute::IBusAttribute;

extern "C" {
    #[doc = " ibus_attr_list_new:\n\n Creates an new #IBusAttrList.\n\n Returns: A newly allocated #IBusAttrList."]
    pub fn ibus_attr_list_new() -> *mut IBusAttrList;
    #[doc = " ibus_attr_list_append:\n @attr_list: An IBusAttrList instance.\n @attr: The IBusAttribute instance to be appended.\n\n Append an IBusAttribute to IBusAttrList, and increase reference."]
    pub fn ibus_attr_list_append(attr_list: *mut IBusAttrList, attr: *mut IBusAttribute);
}

pub type IBusAttrList = [u64; 7usize];
