extern crate cc;


fn main() {
    println!("cargo:rustc-link-lib=marisa");

    cc::Build::new()
        .file("src/wrapper.cpp")
        //.flag("-std=c++17")
        .include("src")
        .compile("wrapper");
}
