// ---------------------------------------------------
// low level C wrappers
// ---------------------------------------------------

use std::ffi::CStr;
use std::os::raw::c_void;

use tempfile::NamedTempFile;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct marisa_obj {
    trie: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct marisa_keyset {
    keyset: *mut c_void,
}

pub type marisa_callback =
unsafe extern "C" fn(user_data: *mut c_void, key: *const u8, len: usize, id: usize) -> bool;

extern "C" {
    fn marisa_new() -> *mut marisa_obj;
    fn marisa_release(self_: *mut marisa_obj);
    fn marisa_build(self_: *mut marisa_obj, keyset: *mut marisa_keyset);
    fn marisa_load(self_: *mut marisa_obj, filename: *const u8);
    fn marisa_save(self_: *mut marisa_obj, filename: *const u8);
    fn marisa_predictive_search(
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
}

// ---------------------------------------------------
// high level API
// ---------------------------------------------------

pub type PredictiveSearchCallback = dyn FnMut(
    &[u8],
    usize) -> bool;

struct Marisa {
    marisa: *mut marisa_obj,
}

impl Marisa {
    pub unsafe fn new() -> Marisa {
        let marisa = marisa_new();
        Marisa { marisa }
    }

    pub unsafe fn load(&self, filename: &String) {
        marisa_load(self.marisa, filename.as_ptr());
    }

    pub unsafe fn build(&self, keyset: &Keyset) {
        marisa_build(self.marisa, keyset.keyset);
    }

    pub unsafe fn save(&self, filename: &String) {
        marisa_save(self.marisa, filename.as_ptr());
    }

    pub unsafe fn num_keys(&self) -> usize {
        marisa_num_keys(self.marisa)
    }

    unsafe extern "C" fn trampoline<F>(
        cookie: *mut c_void,
        s: *const u8,
        len: usize,
        id: usize,
    ) -> bool
        where
            F: FnMut(
                &[u8],
                usize) -> bool,
    {
        let cookie = &mut *(cookie as *mut F);
        let cs = std::slice::from_raw_parts(s, len as usize);
        cookie(cs, id)
    }


    fn get_trampoline<F>(_closure: &F) -> marisa_callback
        where F: FnMut(&[u8], usize) -> bool {
        Marisa::trampoline::<F>
    }

    pub unsafe fn predictive_search<F>(&self, query: &[u8], callback: F)
        where F: FnMut(&[u8], usize) -> bool {
        let mut closure = callback;
        let cb = Marisa::get_trampoline(&closure);
        marisa_predictive_search(
            self.marisa,
            query.as_ptr(),
            query.len(),
            &mut closure as *mut _ as *mut c_void,
            cb,
        );
    }
}

pub struct Keyset {
    keyset: *mut marisa_keyset,
}

impl Keyset {
    pub unsafe fn new() -> Keyset {
        Keyset { keyset: marisa_keyset_new() }
    }
    pub unsafe fn push_back(&self, key: &[u8]) {
        marisa_keyset_push_back(self.keyset, key.as_ptr(), key.len());
    }
}

impl Drop for Keyset {
    fn drop(&mut self) {
        unsafe {
            marisa_keyset_release(self.keyset);
        }
    }
}


#[test]
fn test() {
    let tmpfile = NamedTempFile::new().unwrap();
    let tmpfile = tmpfile.path().to_str().unwrap().to_string();
    // let tmpfile = "/tmp/test.trie".to_string();

    unsafe {
        let keyset = Keyset::new();
        keyset.push_back("apple".as_bytes());
        keyset.push_back("age".as_bytes());
        keyset.push_back("hola".as_bytes());
        let marisa = Marisa::new();
        marisa.build(&keyset);
        marisa.save(&tmpfile);

        assert_eq!(marisa.num_keys(), 3)
    }

    // read it
    unsafe {
        let marisa = Marisa::new();
        marisa.load(&tmpfile);
        assert_eq!(marisa.num_keys(), 3);

        let mut i = 0;
        let mut got: Vec<(String, usize)> = Vec::new();
        assert_eq!("a".as_bytes().len(), 1);

        marisa.predictive_search("a".as_bytes(), |bytes, id| {
            i += 1;
            let key = CStr::from_bytes_with_nul(&*[bytes, b"\0"].concat()).unwrap()
                .to_str().unwrap().to_string();
            got.push((key, id));
            true
        });
        assert_eq!(i, 2);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "age");
        assert_eq!(got[1].0, "apple");
    }
}