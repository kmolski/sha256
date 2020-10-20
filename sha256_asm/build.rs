use std::env;
use std::process::Command;

fn main() {
    // Rebuild if src/sha256sum.s is changed.
    println!("cargo:rerun-if-changed=src/sha256sum.s");

    let out_dir = env::var("OUT_DIR").unwrap();
    if !(Command::new("gcc")
        .args(&[
            "-c",
            "-o",
            &(out_dir.clone() + "/libsha256.a"),
            "src/sha256_asm.s",
        ])
        .status()
        .unwrap()
        .success())
    {
        panic!("failed to build sha256sum.s!");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=sha256");
}
