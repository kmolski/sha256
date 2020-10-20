extern "C" {
    fn asm() -> usize;
}

#[test]
fn test_asm() {
    assert!(1 == unsafe { asm() });
}
