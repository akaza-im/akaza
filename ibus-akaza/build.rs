use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn pkgconfig(module: &str, flag: &str) -> Vec<String> {
    let output = Command::new("pkg-config")
        .arg(module)
        .arg(flag)
        .output()
        .expect("Failed to execute pkg-config");
    let buf = String::from_utf8(output.stdout).expect("Invalid UTF-8 from pkg-config");
    buf.trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn main() {
    // config.h を生成する (Makefile の `config.h: config.h.in` 相当)
    let datadir = env::var("DATADIR").unwrap_or_else(|_| "/usr/share".to_string());
    let config_h = format!("#define PKGDATADIR \"{datadir}/ibus-akaza/\"\n");
    let config_h_path = Path::new("config.h");
    // 内容が変わった場合のみ書き込み、不要な再ビルドを防ぐ
    let needs_write = fs::read_to_string(config_h_path)
        .map(|existing| existing != config_h)
        .unwrap_or(true);
    if needs_write {
        fs::write(config_h_path, &config_h).expect("Failed to write config.h");
    }
    println!("cargo:rerun-if-env-changed=DATADIR");
    println!("cargo:rerun-if-changed=config.h.in");

    println!("cargo:rustc-link-lib=ibus-1.0");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut p = cc::Build::new();
    let mut c = p.file("wrapper.c");
    c = c.include(".");

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
