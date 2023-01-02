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
    println!("cargo:rerun-if-changed=src/fcitx5-akaza.cc");

    let mut p = cc::Build::new();
    let mut c = p.file("src/fcitx5-akaza.cc");
    c = c.include("wrapper")
        .cpp(true)
        .flag("-std=c++17")
        .flag("-g");

    for module in &["Fcitx5Core"] {
        for flag in pkgconfig(module, "--cflags") {
            c = c.flag(flag.as_str());
        }
        for flag in pkgconfig(module, "--libs") {
            println!("cargo:rustc-link-arg={}", flag);
        }
    }
    p.compile("fcitx5-akaza2");
}
