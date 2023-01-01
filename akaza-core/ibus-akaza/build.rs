extern crate cc;

fn main() {
    cc::Build::new()
        .file("wrapper.c")
        .include("wrapper")
        .compile("wrapper");

    println!("cargo:rustc-link-lib=ibus-1.0");
}
