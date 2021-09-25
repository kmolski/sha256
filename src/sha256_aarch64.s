    .text

.macro sha256_round a, b, c, d, e, f, g, h, i
    mov eax, \e
    ror eax, 6
    mov edx, \e
    ror edx, 11
    xor eax, edx
    mov ebx, \e
    ror ebx, 25
    xor eax, ebx
    # eax = sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25)

    mov ebx, \e
    and ebx, \f
    mov edx, \e
    not edx
    and edx, \g
    xor ebx, edx
    # ebx = choice = (e & f) ^ (g & !e)
    add \h, eax
    add \h, ebx
    lea rdx, [rip + ROUND_VALUES]
    add \h, dword ptr [rdx + rcx * 4 + \i]
    add \h, dword ptr [rsi + rcx * 4 + \i]
    # \h = temp1 = h + sigma1 + choice + ROUND_VALUES[i] + w[i]
    add \d, \h
    # d = d + temp1

    mov eax, \a
    ror eax, 2
    mov edx, \a
    ror edx, 13
    xor eax, edx
    mov ebx, \a
    ror ebx, 22
    xor eax, ebx
    # eax = sigma0 = state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22)
    mov edx, \a
    and edx, \b
    # edx = a & b
    mov ebx, \a
    and ebx, \c
    xor edx, ebx
    # edx = (a & b) ^ (a & c)
    mov ebx, \b
    and ebx, \c
    xor edx, ebx
    # edx = majority = (a & b) ^ (a & c) ^ (b & c)
    add eax, edx
    # eax = temp2 = sigma0 + majority
    add \h, eax
    # h = temp1 + temp2
.endm

.global sha256_asm
sha256_asm:
    # arguments - rdi = state: *mut u32, rsi = w: *const u32
    push r15
    push r14
    push r13
    push r12
    push rbx
    mov rcx, 0

    mov r8d,  dword ptr [rdi + 0]
    mov r9d,  dword ptr [rdi + 4]
    mov r10d, dword ptr [rdi + 8]
    mov r11d, dword ptr [rdi + 12]
    mov r12d, dword ptr [rdi + 16]
    mov r13d, dword ptr [rdi + 20]
    mov r14d, dword ptr [rdi + 24]
    mov r15d, dword ptr [rdi + 28]
sha256_asm_loop_start:
    sha256_round r8d,  r9d,  r10d, r11d, r12d, r13d, r14d, r15d, 0
    sha256_round r15d, r8d,  r9d,  r10d, r11d, r12d, r13d, r14d, 4
    sha256_round r14d, r15d, r8d,  r9d,  r10d, r11d, r12d, r13d, 8
    sha256_round r13d, r14d, r15d, r8d,  r9d,  r10d, r11d, r12d, 12
    sha256_round r12d, r13d, r14d, r15d, r8d,  r9d,  r10d, r11d, 16
    sha256_round r11d, r12d, r13d, r14d, r15d, r8d,  r9d,  r10d, 20
    sha256_round r10d, r11d, r12d, r13d, r14d, r15d, r8d,  r9d,  24
    sha256_round r9d,  r10d, r11d, r12d, r13d, r14d, r15d, r8d,  28

    add rcx, 8
    cmp rcx, 64
    jne sha256_asm_loop_start

    mov dword ptr [rdi + 28], r15d
    mov dword ptr [rdi + 24], r14d
    mov dword ptr [rdi + 20], r13d
    mov dword ptr [rdi + 16], r12d
    mov dword ptr [rdi + 12], r11d
    mov dword ptr [rdi + 8],  r10d
    mov dword ptr [rdi + 4],  r9d
    mov dword ptr [rdi + 0],  r8d

    pop rbx
    pop r12
    pop r13
    pop r14
    pop r15
    ret

    .section .rodata

    # The following round constants were taken from:
    # https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11)
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

.balign 32
INDEX_VECTOR:
    .long 7,0,1,2,3,4,5,6
