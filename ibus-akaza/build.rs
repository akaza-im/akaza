extern crate cc;

use std::io::Read;
use std::process::{Command, Stdio};

fn pkgconfig(module: &str, flag: &str) -> Vec<String> {
    let child = Command::new("pkg-config")
        .arg(module)
        .arg(flag)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let mut buf = String::new();
    child.stdout.unwrap().read_to_string(&mut buf).unwrap();
    let args: Vec<&str> = buf.trim().split(' ').collect();
    args.iter().map(|f| f.to_string()).collect()
}

fn main() {
    println!("cargo:rustc-link-lib=ibus-1.0");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut p = cc::Build::new();
    let mut c = p.file("wrapper.c");
    c = c.include("wrapper");

    // Normally, I dislike following options.
    // But, it's a temporary code.
    // TODO: remove following options.
    c = c.flag("-Wno-unused-parameter");
    c = c.flag("-Wno-sign-compare");
    c = c.flag("-Wno-incompatible-pointer-types");

    {
        let module = &"ibus-1.0";
        for flag in pkgconfig(module, "--cflags") {
            c = c.flag(flag.as_str());
        }
        for flag in pkgconfig(module, "--libs") {
            println!("cargo:rustc-link-arg={flag}");
        }
    }
    p.compile("wrapper");
}
