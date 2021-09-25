use std::env;
use std::process::Command;

fn main() {
    // Rebuild if src/sha256_X.s is changed.
    println!("cargo:rerun-if-changed=src/sha256_{}.s", env::consts::ARCH);

    let out_dir = env::var("OUT_DIR").unwrap();
    if !(Command::new("gcc")
        .args(&[
            "-c",
            "-fPIC",
            "-o",
            &(out_dir.clone() + "/libsha256_asm.a"),
            &format!("src/sha256_{}.s", env::consts::ARCH),
        ])
        .status()
        .unwrap()
        .success())
    {
        panic!("failed to build sha256_{}.s!", env::consts::ARCH);
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=sha256_asm");
}
