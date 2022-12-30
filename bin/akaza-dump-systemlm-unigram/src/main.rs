use std::env;
use std::ffi::{CStr, CString};
use marisa_sys::root;

struct SystemUnigramLM {
    trie_: root::marisa::Trie
}

impl SystemUnigramLM {
    pub unsafe fn new() -> Self {
        let trie_ = root::marisa::Trie::new();
        Self { trie_ }
    }

    unsafe fn load(&mut self, path:&String) {
        println!("LOAD {}", path);
        let s = CString::new(path.clone()).unwrap();
        self.trie_.load(s.as_ptr());
    }
    unsafe fn dump(&self) {
        println!("SIZE={}", self.trie_.size());

        let mut agent = root::marisa::Agent::new();
        println!("START");
        let query = CString::new("").unwrap();
        agent.set_query(query.as_ptr());
        println!("START");
        while self.trie_.predictive_search(&mut agent) {
            // うーん。ここでつまづくとは。。C++ 側でちょっとしたラッパー関数を作って
            // marisa_agent_key(agent) { return agent.key() } みたいなのを
            // 入れる必要がありそう。
            println!("START");
            let mut key = agent.key();
            let mut ptr = key.as_ref().expect("A").ptr();
            let str = CStr::from_ptr(ptr).to_bytes();
            println!("KEY={:?}", str);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let mut lm = unsafe { SystemUnigramLM::new() };
    unsafe { lm.load(path); }
    unsafe { lm.dump(); }
}

