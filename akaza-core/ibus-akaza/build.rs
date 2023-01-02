extern crate cc;

use std::io::Read;
use std::process::{Command, Stdio};

fn pkgconfig(module: &str) -> Vec<String> {
    let child = Command::new("pkg-config")
        .arg(module)
        .arg("--cflags")
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
    println!("cargo:rustc-link-lib=enchant-2");
    println!("cargo:rerun-if-changed=wrapper.c");

    let mut p = cc::Build::new();
    let mut c = p.file("wrapper.c");
    c = c.include("wrapper");
    for module in &["ibus-1.0", "enchant-2"] {
        for flag in pkgconfig(module) {
            c = c.flag(flag.as_str());
        }
    }
    p.compile("wrapper");
}
