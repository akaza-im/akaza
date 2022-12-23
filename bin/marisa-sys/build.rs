use std::env;
use std::path::PathBuf;
extern crate cc;


fn main() {
    println!("cargo:rustc-link-lib=marisa");


    cc::Build::new()
        .file("wrapper.cpp")
        //.flag("-std=c++17")
        .include("src")
        .compile("wrapper");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .enable_cxx_namespaces()
        .allowlist_recursively(true)
        .allowlist_function("marisa.*")
        .allowlist_type("marisa.*")
        .allowlist_function(".*Trie.*")
        .allowlist_type(".*Trie.*")
        .allowlist_file(".*marisa.h.*")
        .allowlist_file(".*key.h.*")
        .allowlist_file("key")
        .generate_inline_functions(true)
        .opaque_type("std::.*")
        .generate()
        .expect("Unable to generate bindings!");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
