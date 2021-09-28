    .text

.macro sha256_round a, b, c, d, e, f, g, h, i
    mov w11, \e, ror #6
    eor w11, w11, \e, ror #11
    eor w11, w11, \e, ror #25
    // w11 = sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25)

    and w12, \e, \f
    mvn w13, \e
    and w13, w13, \g
    eor w12, w12, w13
    // w12 = choice = (e & f) ^ (g & !e)
    add \h, \h, w11
    add \h, \h, w12
    // \h = h + sigma1 + choice
    add x15, x2, x14
    ldr w11, [x15, #\i]
    add \h, \h, w11
    add x16, x2, x1
    ldr w12, [x16, #\i]
    add \h, \h, w12
    // \h = temp1 = h + sigma1 + choice + ROUND_VALUES[i] + w[i]
    add \d, \d, \h
    // d = d + temp1

    mov w11, \a, ror #2
    eor w11, w11, \a, ror #13
    eor w11, w11, \a, ror #22
    // w11 = sigma0 = state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22)
    and w12, \a, \b
    and w13, \a, \c
    eor w12, w12, w13
    // w12 = (a & b) ^ (a & c)
    and w13, \b, \c
    eor w12, w12, w13
    // w12 = majority = (a & b) ^ (a & c) ^ (b & c)
    add w11, w11, w12
    // w11 = temp2 = sigma0 + majority
    add \h, \h, w11
    // h = temp1 + temp2
.endm

.global sha256_asm
sha256_asm:
    // arguments - x0 = state: *mut u32, x1 = w: *const u32
    mov x2, xzr
    adrp x14, ROUND_VALUES
    add x14, x14, :lo12:ROUND_VALUES

    ldp w3, w4,  [x0, #0]
    ldp w5, w6,  [x0, #8]
    ldp w7, w8,  [x0, #16]
    ldp w9, w10, [x0, #24]
sha256_asm_loop_start:
    sha256_round w3,  w4,  w5,  w6,  w7,  w8,  w9,  w10, 0
    sha256_round w10, w3,  w4,  w5,  w6,  w7,  w8,  w9,  4
    sha256_round w9,  w10, w3,  w4,  w5,  w6,  w7,  w8,  8
    sha256_round w8,  w9,  w10, w3,  w4,  w5,  w6,  w7,  12
    sha256_round w7,  w8,  w9,  w10, w3,  w4,  w5,  w6,  16
    sha256_round w6,  w7,  w8,  w9,  w10, w3,  w4,  w5,  20
    sha256_round w5,  w6,  w7,  w8,  w9,  w10, w3,  w4,  24
    sha256_round w4,  w5,  w6,  w7,  w8,  w9,  w10, w3,  28

    add x2, x2, #32
    cmp x2, #256
    b.ne sha256_asm_loop_start

    stp w9, w10, [x0, #24]
    stp w7, w8,  [x0, #16]
    stp w5, w6,  [x0, #8]
    stp w3, w4,  [x0, #0]
    ret

    .section .rodata

    // The following round constants were taken from:
    // https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11)
.balign 4
ROUND_VALUES:
    .long 0x428A2F98,0x71374491,0xB5C0FBCF,0xE9B5DBA5,0x3956C25B,0x59F111F1,0x923F82A4,0xAB1C5ED5
    .long 0xD807AA98,0x12835B01,0x243185BE,0x550C7DC3,0x72BE5D74,0x80DEB1FE,0x9BDC06A7,0xC19BF174
    .long 0xE49B69C1,0xEFBE4786,0x0FC19DC6,0x240CA1CC,0x2DE92C6F,0x4A7484AA,0x5CB0A9DC,0x76F988DA
    .long 0x983E5152,0xA831C66D,0xB00327C8,0xBF597FC7,0xC6E00BF3,0xD5A79147,0x06CA6351,0x14292967
    .long 0x27B70A85,0x2E1B2138,0x4D2C6DFC,0x53380D13,0x650A7354,0x766A0ABB,0x81C2C92E,0x92722C85
    .long 0xA2BFE8A1,0xA81A664B,0xC24B8B70,0xC76C51A3,0xD192E819,0xD6990624,0xF40E3585,0x106AA070
    .long 0x19A4C116,0x1E376C08,0x2748774C,0x34B0BCB5,0x391C0CB3,0x4ED8AA4A,0x5B9CCA4F,0x682E6FF3
    .long 0x748F82EE,0x78A5636F,0x84C87814,0x8CC70208,0x90BEFFFA,0xA4506CEB,0xBEF9A3F7,0xC67178F2
