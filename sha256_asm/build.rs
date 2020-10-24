use std::env;
use std::process::Command;

fn main() {
    // Rebuild if src/sha256_asm.s is changed.
    println!("cargo:rerun-if-changed=src/sha256_asm.s");

    let out_dir = env::var("OUT_DIR").unwrap();
    if !(Command::new("gcc")
        .args(&[
            "-c",
            "-fPIC",
            "-o",
            &(out_dir.clone() + "/libsha256_asm.a"),
            "src/sha256_asm.s",
        ])
        .status()
        .unwrap()
        .success())
    {
        panic!("failed to build sha256_asm.s!");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=sha256_asm");
}
