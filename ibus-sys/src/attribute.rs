use crate::glib::guint;

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
#[doc = " IBusAttribute:\n @type: IBusAttributeType\n @value: Value for the type.\n @start_index: The starting index, inclusive.\n @end_index: The ending index, exclusive.\n\n Signify the type, value and scope of the attribute.\n The scope starts from @start_index till the @end_index-1."]
pub type IBusAttribute = [u64; 8usize];

extern "C" {
    // attribute
    #[doc = " ibus_attribute_new:\n @type: Type of the attribute.\n @value: Value of the attribute.\n @start_index: Where attribute starts.\n @end_index: Where attribute ends.\n\n Creates a new IBusAttribute.\n\n Returns: (transfer none): A newly allocated IBusAttribute."]
    pub fn ibus_attribute_new(
        type_: guint,
        value: guint,
        start_index: guint,
        end_index: guint,
    ) -> *mut IBusAttribute;
}
