    .intel_syntax noprefix

    .text

.global sha256_asm
sha256_asm:
    # arguments - rdi = temp: *mut u32, rsi = w: *const u32
    push r15
    push r14
    push r13
    push r12
    mov rcx, 0
loop:   
    # let s0 = temp[0].rotate_right(2) ^ temp[0].rotate_right(13) ^ temp[0].rotate_right(22)
    mov r8d, dword ptr [rdi + 0]
    # r8d = temp[0]
    rorx eax, r8d, 2
    # eax = temp[0].rotate_right(2)
    rorx r9d, r8d, 13
    xor eax, r9d
    # eax = temp[0].rotate_right(2) ^ temp[0].rotate_right(13)
    rorx r10d, r8d, 22
    xor eax, r10d
    # eax  = s0 = temp[0].rotate_right(2) ^ temp[0].rotate_right(13) ^ temp[0].rotate_right(22)
    mov r11d, eax
    # r11d = s0 = temp[0].rotate_right(2) ^ temp[0].rotate_right(13) ^ temp[0].rotate_right(22)

    # let maj = (temp[0] & temp[1]) ^ (temp[0] & temp[2]) ^ (temp[1] & temp[2])
    mov r9d,  dword ptr [rdi + 4]
    mov r10d, dword ptr [rdi + 8]
    # r8d = temp[0], r9d = temp[1], r10d = temp[2]
    mov eax, r8d
    and eax, r9d
    # eax = (temp[0] & temp[1])
    and r8d, r10d
    # r8d = (temp[0] & temp[2])
    and r9d, r10d
    # r9d = (temp[1] & temp[2])
    xor eax, r8d
    xor eax, r9d
    # eax = maj = (temp[0] & temp[1]) ^ (temp[0] & temp[2]) ^ (temp[1] & temp[2])

    # let temp2 = Wrap(s0) + Wrap(maj)
    add eax, r11d
    # eax = temp2 = Wrap(s0) + Wrap(maj) 
    mov r11d, dword ptr [rdi + 28]
    mov dword ptr [rdi + 28], eax
    mov eax, r11d
    # eax = temp[7], temp[7] = temp2

    lea rdx, [rip + ROUND_VALUES]
    add eax, dword ptr [rdx + rcx * 4]
    # eax = temp[7] + ROUND_VALUES[i]
    add eax, dword ptr [rsi + rcx * 4]
    # eax = temp[7] + ROUND_VALUES[i] + w[i]
    
    # let s1 = temp[4].rotate_right(6) ^ temp[4].rotate_right(11) ^ temp[4].rotate_right(25)
    mov r10d, dword ptr [rdi + 16]
    # r10d = temp[4]
    rorx r9d, r10d, 6
    # r9d = temp[4].rotate_right(6)
    rorx r8d, r10d, 11
    xor r9d, r8d
    # r9d = temp[4].rotate_right(6) ^ temp[4].rotate_right(11)
    rorx r11d, r10d, 25
    xor r9d, r11d
    # r9d = s1 = temp[4].rotate_right(6) ^ temp[4].rotate_right(11) ^ temp[4].rotate_right(25)
    add eax, r9d
    # eax = temp[7] + ROUND_VALUES[i] + w[i] + s1
    
    # let ch = (temp[4] & temp[5]) ^ (!temp[4] & temp[6])
    mov r9d, dword ptr [rdi + 20]
    # r9d = temp[5]
    and r9d, r10d
    # r9d = (temp[4] & temp[5])
    mov r11d, dword ptr [rdi + 24]
    # r11d = temp[6]
    andn r11d, r10d, r11d
    # r11d = (!temp[4] & temp[6])
    xor r9d, r11d
    # r9d = (temp[4] & temp[5]) ^ (!temp[4] & temp[6])
    add eax, r9d
    # eax = temp1 = Wrap(temp[7]) + Wrap(s1) + Wrap(ch) + Wrap(ROUND_VALUES[i]) + Wrap(w[i]);

    mov r15d, dword ptr [rdi + 28]
    mov r14d, dword ptr [rdi + 24]
    mov r13d, dword ptr [rdi + 20]
    mov r12d, dword ptr [rdi + 16]
    mov r11d, dword ptr [rdi + 12]
    mov r10d, dword ptr [rdi +  8]
    mov r9d,  dword ptr [rdi +  4]
    mov r8d,  dword ptr [rdi +  0]

    mov dword ptr [rdi + 28], r14d
    mov dword ptr [rdi + 24], r13d
    mov dword ptr [rdi + 20], r12d
    mov dword ptr [rdi + 16], r11d
    mov dword ptr [rdi + 12], r10d
    mov dword ptr [rdi +  8], r9d
    mov dword ptr [rdi +  4], r8d
    mov dword ptr [rdi +  0], r15d

    add dword ptr [rdi +  0], eax
    add dword ptr [rdi + 16], eax

    # temp[7] = temp[6]
    # temp[6] = temp[5]
    # temp[5] = temp[4]
    # temp[4] = (Wrap(temp[3]) + temp1).0
    # temp[3] = temp[2]
    # temp[2] = temp[1]
    # temp[1] = temp[0]
    # temp[0] = (temp1 + temp2).0

    # mov ymm2, [rdi + 0]
    # # ymm2 = temp
    # vpermd ymm0, ymm1, ymm2
    # # ymm0 = temp.rotate_left(1)
    # 
    # xor ymm2, ymm2
    # mov xmm1, eax
    
    # mov [rdi + 0], ymm0
    inc rcx
    cmp rcx, 64
    jne loop
    pop r12
    pop r13
    pop r14
    pop r15
    ret

    .data

    # The following round constants were taken from:
    # https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11 and 15)
.align 64
ROUND_VALUES:
    .long 0x428A2F98,0x71374491,0xB5C0FBCF,0xE9B5DBA5,0x3956C25B,0x59F111F1,0x923F82A4,0xAB1C5ED5
    .long 0xD807AA98,0x12835B01,0x243185BE,0x550C7DC3,0x72BE5D74,0x80DEB1FE,0x9BDC06A7,0xC19BF174
    .long 0xE49B69C1,0xEFBE4786,0x0FC19DC6,0x240CA1CC,0x2DE92C6F,0x4A7484AA,0x5CB0A9DC,0x76F988DA
    .long 0x983E5152,0xA831C66D,0xB00327C8,0xBF597FC7,0xC6E00BF3,0xD5A79147,0x06CA6351,0x14292967
    .long 0x27B70A85,0x2E1B2138,0x4D2C6DFC,0x53380D13,0x650A7354,0x766A0ABB,0x81C2C92E,0x92722C85
    .long 0xA2BFE8A1,0xA81A664B,0xC24B8B70,0xC76C51A3,0xD192E819,0xD6990624,0xF40E3585,0x106AA070
    .long 0x19A4C116,0x1E376C08,0x2748774C,0x34B0BCB5,0x391C0CB3,0x4ED8AA4A,0x5B9CCA4F,0x682E6FF3
    .long 0x748F82EE,0x78A5636F,0x84C87814,0x8CC70208,0x90BEFFFA,0xA4506CEB,0xBEF9A3F7,0xC67178F2
