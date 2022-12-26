extern crate cc;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("wrapper.cc")
        .include("wrapper")
        .compile("wrapper");

    println!("cargo:rustc-link-lib=marisa");
}
