extern crate cc;

fn main() {
    cc::Build::new()
        .file("rx/rx.c")
        .include("rx")
        .compile("rx");
}
