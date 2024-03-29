// ---------------------------------------------------
// low level C wrappers
// ---------------------------------------------------

use std::ffi::{c_char, CString};
use std::os::raw::c_void;

use anyhow::{anyhow, Result};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct marisa_obj {
    trie: *mut c_void,
}

// ↓↓ It's unsafe operation. I'll remove this in the future.
unsafe impl Send for marisa_obj {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct marisa_keyset {
    keyset: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct marisa_exception {
    error_message: *mut c_char,
}

pub type marisa_callback =
    unsafe extern "C" fn(user_data: *mut c_void, key: *const u8, len: usize, id: usize) -> bool;

extern "C" {
    fn marisa_new() -> *mut marisa_obj;
    fn marisa_release(self_: *mut marisa_obj);
    fn marisa_build(self_: *mut marisa_obj, keyset: *mut marisa_keyset);
    fn marisa_load(self_: *mut marisa_obj, filename: *const c_char) -> *mut marisa_exception;
    fn marisa_save(self_: *mut marisa_obj, filename: *const c_char) -> *mut marisa_exception;
    fn marisa_predictive_search(
        self_: *mut marisa_obj,
        query: *const u8,
        query_len: usize,
        user_data: *mut c_void,
        cb: marisa_callback,
    );
    fn marisa_common_prefix_search(
        self_: *mut marisa_obj,
        query: *const u8,
        query_len: usize,
        user_data: *mut c_void,
        cb: marisa_callback,
    );
    fn marisa_num_keys(self_: *mut marisa_obj) -> usize;

    fn marisa_keyset_new() -> *mut marisa_keyset;
    fn marisa_keyset_push_back(self_: *mut marisa_keyset, ptr: *const u8, len: usize);
    fn marisa_keyset_release(self_: *mut marisa_keyset);

    fn marisa_exception_release(self_: *mut marisa_exception);
}

// ---------------------------------------------------
// high level API
// ---------------------------------------------------

pub type PredictiveSearchCallback = dyn FnMut(&[u8], usize) -> bool;

pub struct Marisa {
    marisa: *mut marisa_obj,
}

impl Default for Marisa {
    fn default() -> Marisa {
        let marisa = unsafe { marisa_new() };
        Marisa { marisa }
    }
}

impl Marisa {
    pub fn load(&mut self, filename: &str) -> Result<()> {
        unsafe {
            let cstring = CString::new(filename).unwrap();
            let exc = marisa_load(self.marisa, cstring.as_ptr());
            if exc.is_null() {
                Ok(())
            } else {
                Err(anyhow!(
                    "Cannot load file: {}, file={}",
                    CString::from_raw((*exc).error_message)
                        .into_string()
                        .unwrap(),
                    filename
                ))
            }
        }
    }

    pub fn build(&mut self, keyset: &Keyset) {
        unsafe {
            marisa_build(self.marisa, keyset.keyset);
        }
    }

    pub fn save(&self, filename: &str) -> Result<()> {
        unsafe {
            let cstring = CString::new(filename).unwrap();
            let exc = marisa_save(self.marisa, cstring.as_ptr());
            if exc.is_null() {
                Ok(())
            } else {
                Err(anyhow!(
                    "Cannot save marisa file: {}, filename={}",
                    CString::from_raw((*exc).error_message)
                        .into_string()
                        .unwrap(),
                    filename
                ))
            }
        }
    }

    pub fn num_keys(&self) -> usize {
        unsafe { marisa_num_keys(self.marisa) }
    }

    unsafe extern "C" fn trampoline<F>(
        cookie: *mut c_void,
        s: *const u8,
        len: usize,
        id: usize,
    ) -> bool
    where
        F: FnMut(&[u8], usize) -> bool,
    {
        let cookie = &mut *(cookie as *mut F);
        let cs = std::slice::from_raw_parts(s, len);
        cookie(cs, id)
    }

    fn get_trampoline<F>(_closure: &F) -> marisa_callback
    where
        F: FnMut(&[u8], usize) -> bool,
    {
        Marisa::trampoline::<F>
    }

    pub fn predictive_search<F>(&self, query: &[u8], callback: F)
    where
        F: FnMut(&[u8], usize) -> bool,
    {
        let mut closure = callback;
        let cb = Marisa::get_trampoline(&closure);
        unsafe {
            marisa_predictive_search(
                self.marisa,
                query.as_ptr(),
                query.len(),
                &mut closure as *mut _ as *mut c_void,
                cb,
            );
        }
    }

    pub fn common_prefix_search<F>(&self, query: &str, callback: F)
    where
        F: FnMut(&[u8], usize) -> bool,
    {
        let mut closure = callback;
        let cb = Marisa::get_trampoline(&closure);
        unsafe {
            marisa_common_prefix_search(
                self.marisa,
                query.as_ptr(),
                query.len(),
                &mut closure as *mut _ as *mut c_void,
                cb,
            );
        }
    }
}

pub struct Keyset {
    keyset: *mut marisa_keyset,
}

impl Default for Keyset {
    fn default() -> Self {
        unsafe {
            Keyset {
                keyset: marisa_keyset_new(),
            }
        }
    }
}

impl Keyset {
    pub fn push_back(&mut self, key: &[u8]) {
        unsafe {
            marisa_keyset_push_back(self.keyset, key.as_ptr(), key.len());
        }
    }
}

impl Drop for Keyset {
    fn drop(&mut self) {
        unsafe {
            marisa_keyset_release(self.keyset);
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::{Keyset, Marisa};

    #[test]
    fn test() {
        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfile = tmpfile.path().to_str().unwrap().to_string();
        // let tmpfile = "/tmp/test.trie".to_string();

        {
            let mut keyset = Keyset::default();
            keyset.push_back("apple".as_bytes());
            keyset.push_back("age".as_bytes());
            keyset.push_back("hola".as_bytes());
            let mut marisa = Marisa::default();
            marisa.build(&keyset);
            marisa.save(&tmpfile).unwrap();

            assert_eq!(marisa.num_keys(), 3)
        }

        // read it
        {
            let mut marisa = Marisa::default();
            marisa.load(&tmpfile).unwrap();
            assert_eq!(marisa.num_keys(), 3);

            let mut i = 0;
            let mut got: Vec<(String, usize)> = Vec::new();
            assert_eq!("a".as_bytes().len(), 1);

            marisa.predictive_search("a".as_bytes(), |bytes, id| {
                i += 1;
                let key = String::from_utf8(bytes.to_vec()).unwrap();
                got.push((key, id));
                true
            });
            assert_eq!(i, 2);
            assert_eq!(got.len(), 2);
            assert_eq!(got[0].0, "age");
            assert_eq!(got[1].0, "apple");
        }
    }

    #[test]
    fn test_exc() {
        {
            let mut marisa = Marisa::default();
            let result = marisa.load("UNKNOWN_PATH");
            if let Err(err) = result {
                assert!(err.to_string().contains("MARISA_IO_"));
            } else {
                panic!() // unreachable
            }
        }
    }
}
