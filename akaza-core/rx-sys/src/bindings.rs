/* BGased on generated code by rust-bindgen 0.63.0 from rx/rx.h */

use std::ffi::{c_int, CStr};
use std::ffi::CString;
use std::os::raw::c_uchar;

pub const RX_SEARCH_DEFAULT: u32 = 0;
pub const RX_SEARCH_PREDICTIVE: u32 = 1;
pub const RX_SEARCH_1LEVEL: u32 = 2;
pub const RX_SEARCH_DEPTH_SHIFT: u32 = 2;
pub const RX_SEARCH_DEPTH_MASK: u32 = 1020;

pub struct RXBuilder {
    rx: *mut rx_builder,
}

impl RXBuilder {
    pub unsafe fn new() -> RXBuilder {
        RXBuilder { rx: rx_builder_create() }
    }

    // key should be c-string.
    // but it maybe, not a utf-8 string.
    pub unsafe fn add(&self, key: Vec<u8>) {
        let p = CString::from_vec_with_nul(key).expect("Cannot convert to CString");
        rx_builder_add(self.rx, p.as_ptr());
    }

    pub unsafe fn get_size(&self) -> i32 {
        return rx_builder_get_size(self.rx);
    }
    pub unsafe fn get_image(&self) -> *mut u8 {
        return rx_builder_get_image(self.rx);
    }

    pub unsafe fn set_bits(&self, bits: i32) {
        rx_builder_set_bits(self.rx, bits);
    }

    pub unsafe fn build(&self) -> i32 {
        return rx_builder_build(self.rx);
    }

    pub unsafe fn get_key_index(&self, str: String) -> i32 {
        let p = CString::new(str).expect("Cannot convert to CString");
        return rx_builder_get_key_index(self.rx, p.as_ptr());
    }

    pub unsafe fn dump(&self) {
        rx_builder_dump(self.rx);
    }
}

impl Drop for RXBuilder {
    fn drop(&mut self) {
        unsafe {
            rx_builder_release(self.rx);
        }
    }
}

pub type SearchCallback = unsafe extern "C" fn(
    *mut ::std::os::raw::c_void,
    *const ::std::os::raw::c_char,
    ::std::os::raw::c_int,
    ::std::os::raw::c_int,
) -> i32;

pub struct Rx {
    rx: *mut rx,
}

impl Rx {
    pub unsafe fn open(ptr: *mut u8) -> Rx {
        Rx { rx: rx_open(ptr) }
    }

    unsafe extern "C" fn trampoline<F>(
        cookie: *mut ::std::os::raw::c_void,
        s: *const ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
        id: ::std::os::raw::c_int,
    ) -> i32
        where
            F: FnMut(
                String,
                ::std::os::raw::c_int,
                ::std::os::raw::c_int) -> i32,
    {
        let cookie = &mut *(cookie as *mut F);
        let cs = CStr::from_ptr(s);
        cookie(cs.to_str().unwrap().to_string(), len, id)
    }


    fn get_trampoline<F>(_closure: &F) -> SearchCallback
        where
            F: FnMut(String,
                ::std::os::raw::c_int,
                ::std::os::raw::c_int) -> i32,
    {
        Rx::trampoline::<F>
    }

    pub unsafe fn search<F>(&self, flags: i32, v: Vec<u8>, callback: F)
        where F: FnMut(String, i32, i32) -> i32 {
        let mut closure = callback;
        let cb = Rx::get_trampoline(&closure);

        let p = CString::from_vec_with_nul(v).unwrap();
        rx_search(
            self.rx,
            flags,
            p.as_ptr(),
            Some(cb),
            &mut closure as *mut _ as *mut ::std::os::raw::c_void,
        );
    }
}

impl Drop for Rx {
    fn drop(&mut self) {
        unsafe {
            rx_close(self.rx);
        }
    }
}

struct RbxBuilder {
    builder: *mut rbx_builder,
}

impl RbxBuilder {
    pub unsafe fn new() -> RbxBuilder {
        RbxBuilder { builder: rbx_builder_create() }
    }
    pub unsafe fn push(&self, bytes: &[u8]) {
        rbx_builder_push(self.builder, bytes.as_ptr(), bytes.len() as c_int)
    }

    pub unsafe fn build(&self) {
        rbx_builder_build(self.builder);
    }

    pub unsafe fn get_image(&self) -> *mut u8 {
        return rbx_builder_get_image(self.builder);
    }

    pub unsafe fn get_size(&self) -> std::os::raw::c_int {
        return rbx_builder_get_size(self.builder);
    }
}

impl Drop for RbxBuilder {
    fn drop(&mut self) {
        unsafe { rbx_builder_release(self.builder) }
    }
}

struct Rbx {
    rbx: *mut rbx,
}

impl Rbx {
    unsafe fn load(image: *const u8) -> Rbx {
        return Rbx { rbx: rbx_open(image) };
    }

    // unsafe fn get(&self, idx: i32) {
    //     let got = rbx_get(self.rbx, idx, len);
    // }
}

impl Drop for Rbx {
    fn drop(&mut self) {
        unsafe {
            rbx_close(self.rbx);
        }
    }
}

// TODO support RBX

// TODO make following parts, private.

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct rx {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct rx_builder {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct rbx {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct rbx_builder {
    _unused: [u8; 0],
}

extern "C" {
    fn rx_builder_create() -> *mut rx_builder;

    fn rx_builder_release(builder: *mut rx_builder);

    fn rx_builder_add(builder: *mut rx_builder, word: *const ::std::os::raw::c_char);

    fn rx_builder_build(builder: *mut rx_builder) -> ::std::os::raw::c_int;

    fn rx_builder_get_image(builder: *mut rx_builder) -> *mut ::std::os::raw::c_uchar;

    fn rx_builder_get_size(builder: *mut rx_builder) -> ::std::os::raw::c_int;

    fn rx_builder_get_key_index(
        builder: *mut rx_builder,
        key: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;

    fn rx_builder_set_bits(builder: *mut rx_builder, bits: ::std::os::raw::c_int);

    fn rx_builder_dump(builder: *mut rx_builder);

    fn rx_open(image: *const ::std::os::raw::c_uchar) -> *mut rx;

    fn rx_close(r: *mut rx);

    fn rx_search(
        r: *const rx,
        flags: ::std::os::raw::c_int,
        s: *const ::std::os::raw::c_char,
        cb: ::std::option::Option<
            unsafe extern "C" fn(
                cookie: *mut ::std::os::raw::c_void,
                s: *const ::std::os::raw::c_char,
                len: ::std::os::raw::c_int,
                id: ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int,
        >,
        cookie: *mut ::std::os::raw::c_void,
    );

    fn rx_search_expand(
        r: *const rx,
        flags: ::std::os::raw::c_int,
        s: *const ::std::os::raw::c_char,
        cb: ::std::option::Option<
            unsafe extern "C" fn(
                cookie: *mut ::std::os::raw::c_void,
                s: *const ::std::os::raw::c_char,
                len: ::std::os::raw::c_int,
                id: ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int,
        >,
        cookie: *mut ::std::os::raw::c_void,
        cb_expand_chars: ::std::option::Option<
            unsafe extern "C" fn(
                expansion_data: *const ::std::os::raw::c_void,
                s: ::std::os::raw::c_char,
            ) -> *const ::std::os::raw::c_char,
        >,
        expansion_data: *const ::std::os::raw::c_void,
    );

    fn rx_reverse(
        r: *const rx,
        n: ::std::os::raw::c_int,
        buf: *mut ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;

    fn rbx_builder_create() -> *mut rbx_builder;

    fn rbx_builder_set_length_coding(
        builder: *mut rbx_builder,
        min: ::std::os::raw::c_int,
        step: ::std::os::raw::c_int,
    );

    fn rbx_builder_push(
        builder: *mut rbx_builder,
        bytes: *const u8,
        len: ::std::os::raw::c_int,
    );

    fn rbx_builder_build(builder: *mut rbx_builder) -> ::std::os::raw::c_int;

    fn rbx_builder_get_image(builder: *mut rbx_builder) -> *mut ::std::os::raw::c_uchar;

    fn rbx_builder_get_size(builder: *mut rbx_builder) -> ::std::os::raw::c_int;

    fn rbx_builder_release(builder: *mut rbx_builder);

    fn rbx_open(image: *const ::std::os::raw::c_uchar) -> *mut rbx;

    fn rbx_close(r: *mut rbx);

    fn rbx_get(
        r: *mut rbx,
        idx: ::std::os::raw::c_int,
        len: *mut ::std::os::raw::c_int,
    ) -> *const ::std::os::raw::c_uchar;
}

#[test]
fn test() {
    unsafe {
        let rx_builder = RXBuilder::new();
        rx_builder.set_bits(8);
        rx_builder.add("apple\0".to_string().into_bytes());
        rx_builder.add("ago\0".to_string().into_bytes());
        rx_builder.add("abc\0".to_string().into_bytes());
        rx_builder.add("quick\0".to_string().into_bytes());
        rx_builder.build();

        assert_eq!(rx_builder.get_size(), 39);
        rx_builder.dump();

        let idx = rx_builder.get_key_index("abc".to_string());
        assert_eq!(idx, 0);
        let idx2 = rx_builder.get_key_index("apple".to_string());
        assert_eq!(idx2, 2);
        let idx3 = rx_builder.get_key_index("UNKNOWN".to_string());
        assert_eq!(idx3, -1);

        let rx = Rx::open(rx_builder.get_image());
        {
            let mut i = 0;
            rx.search(0, "abc\0".to_string().into_bytes(), |s, len, id| {
                println!("s={}, len={}, id={}", s, len, id);
                i += 1;
                /* returns non-zero value if you want to stop more traversal. */
                0
            });
            assert_eq!(i, 1);
        }
        {
            let mut i = 0;
            rx.search(1, "a\0".to_string().into_bytes(), |s, len, id| {
                println!("s={}, len={}, id={}", s, len, id);
                i += 1;
                /* returns non-zero value if you want to stop more traversal. */
                0
            });
            assert_eq!(i, 3);
        }
    }
}
