fn main() {
    println!("cargo:rustc-link-lib=dylib=sha256_asm");
    println!("cargo:rustc-link-lib=dylib=sha256_rust");
}
