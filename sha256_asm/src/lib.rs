extern "C" {
    pub fn sha256_asm(temp: *mut u32, w: *const u32);
}

#[no_mangle]
pub fn sha256_rounds_asm(temp: *mut u32, w: *const u32) {
    unsafe { sha256_asm(temp, w) };
}
