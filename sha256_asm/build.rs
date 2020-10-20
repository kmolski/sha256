use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    if !(Command::new("as")
        .args(&["-o", &(out_dir.clone() + "/a.o"), "src/sha256_asm.s"])
        .status()
        .unwrap()
        .success()
        && Command::new("ar")
            .args(&[
                "-crus",
                &(out_dir.clone() + "/liba.a"),
                &(out_dir.clone() + "/a.o"),
            ])
            .status()
            .unwrap()
            .success())
    {
        panic!("failed to build sha256sum.s!");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=a");
}
