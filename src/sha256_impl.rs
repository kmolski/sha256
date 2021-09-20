use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::mem::size_of;
use std::num::Wrapping as Wrap;

// The following initialization data was taken from:
// https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 15)

const INIT_HASH_VALUES: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const CHUNK_SIZE: usize = 512 / 8;
const CHUNK_MINUS_U64: usize = CHUNK_SIZE - size_of::<u64>();
const HASH_SIZE: usize = 256 / 8;

// The following round constants were taken from:
// https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11)

const ROUND_VALUES: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];

macro_rules! sha256_round {
    ($a: ident, $b: ident, $c: ident, $d: ident, $e: ident, $f: ident, $g: ident, $h: ident, $w: ident, $i: expr) => {
        let sigma1 = $e.rotate_right(6) ^ $e.rotate_right(11) ^ $e.rotate_right(25);
        let choice = ($e & $f) ^ ($g & !$e);
        let temp1 = Wrap($h) + Wrap(sigma1) + Wrap(choice) + Wrap(ROUND_VALUES[$i]) + Wrap($w[$i]);
        let sigma0 = $a.rotate_right(2) ^ $a.rotate_right(13) ^ $a.rotate_right(22);
        let majority = ($a & $b) ^ ($a & $c) ^ ($b & $c);
        let temp2 = Wrap(sigma0) + Wrap(majority);

        $d = (Wrap($d) + temp1).0;
        $h = (temp1 + temp2).0;
    };
}

macro_rules! sha256_eight_rounds {
    ($a: ident, $b: ident, $c: ident, $d: ident, $e: ident, $f: ident, $g: ident, $h: ident, $w: ident, $i: expr) => {
        sha256_round!($a, $b, $c, $d, $e, $f, $g, $h, $w, $i * 8 + 0);
        sha256_round!($h, $a, $b, $c, $d, $e, $f, $g, $w, $i * 8 + 1);
        sha256_round!($g, $h, $a, $b, $c, $d, $e, $f, $w, $i * 8 + 2);
        sha256_round!($f, $g, $h, $a, $b, $c, $d, $e, $w, $i * 8 + 3);
        sha256_round!($e, $f, $g, $h, $a, $b, $c, $d, $w, $i * 8 + 4);
        sha256_round!($d, $e, $f, $g, $h, $a, $b, $c, $w, $i * 8 + 5);
        sha256_round!($c, $d, $e, $f, $g, $h, $a, $b, $w, $i * 8 + 6);
        sha256_round!($b, $c, $d, $e, $f, $g, $h, $a, $w, $i * 8 + 7);
    };
}

pub fn sha256_rounds_rust(state: &mut [u32; 8], w: &[u32; 64]) {
    let (mut a, mut b, mut c, mut d) = (state[0], state[1], state[2], state[3]);
    let (mut e, mut f, mut g, mut h) = (state[4], state[5], state[6], state[7]);

    for i in 0..8 {
        sha256_eight_rounds!(a, b, c, d, e, f, g, h, w, i);
    }

    state.copy_from_slice(&[a, b, c, d, e, f, g, h]);
}

extern "C" {
    pub fn sha256_asm(temp: *mut u32, w: *const u32);
}

pub fn sha256_rounds_asm(temp: &mut [u32; 8], w: &[u32; 64]) {
    unsafe { sha256_asm(temp.as_mut_ptr(), w.as_ptr()) };
}

// The following testing data was taken from:
// https://www.di-mgt.com.au/sha_testvectors.html
// https://www.nist.gov/itl/ssd/software-quality-group/nsrl-test-data

#[test]
fn test_string_hash_1() {
    let msg = "abc";
    let hash = [
        0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE, 0x22,
        0x23, 0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00,
        0x15, 0xAD,
    ];

    let mut ctx = SHA256Context::new(sha256_rounds_rust);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);
}

#[test]
fn test_string_hash_2() {
    let msg = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let hash = [
        0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E, 0x60,
        0x39, 0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4, 0x19, 0xDB,
        0x06, 0xC1,
    ];

    let mut ctx = SHA256Context::new(sha256_rounds_rust);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);
}

#[test]
fn test_string_hash_3() {
    let msg = "a".repeat(1_000_000);
    let hash = [
        0xCD, 0xC7, 0x6E, 0x5C, 0x99, 0x14, 0xFB, 0x92, 0x81, 0xA1, 0xC7, 0xE2, 0x84, 0xD7, 0x3E,
        0x67, 0xF1, 0x80, 0x9A, 0x48, 0xA4, 0x97, 0x20, 0x0E, 0x04, 0x6D, 0x39, 0xCC, 0xC7, 0x11,
        0x2C, 0xD0,
    ];

    let mut ctx = SHA256Context::new(sha256_rounds_rust);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);
}

#[test]
fn test_string_hash_4() {
    let msg = "";
    let hash = [
        0xE3, 0xB0, 0xC4, 0x42, 0x98, 0xFC, 0x1C, 0x14, 0x9A, 0xFB, 0xF4, 0xC8, 0x99, 0x6F, 0xB9,
        0x24, 0x27, 0xAE, 0x41, 0xE4, 0x64, 0x9B, 0x93, 0x4C, 0xA4, 0x95, 0x99, 0x1B, 0x78, 0x52,
        0xB8, 0x55,
    ];

    let mut ctx = SHA256Context::new(sha256_rounds_rust);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);
}

#[test]
fn test_string_hash_5() {
    let msg = "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    let hash = [
        0xCF, 0x5B, 0x16, 0xA7, 0x78, 0xAF, 0x83, 0x80, 0x03, 0x6C, 0xE5, 0x9E, 0x7B, 0x04, 0x92,
        0x37, 0x0B, 0x24, 0x9B, 0x11, 0xE8, 0xF0, 0x7A, 0x51, 0xAF, 0xAC, 0x45, 0x03, 0x7A, 0xFE,
        0xE9, 0xD1,
    ];

    let mut ctx = SHA256Context::new(sha256_rounds_rust);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    assert!(ctx.hash_bytes(msg.as_bytes()) == hash);
}

type RoundsFn = fn(&mut [u32; 8], &[u32; 64]);

pub struct SHA256Context {
    state: [u32; 8], // State vector (256 bits)
    data_len: usize, // Data length in bits
    rounds_fn: RoundsFn,
}

impl SHA256Context {
    pub fn new(rounds_fn: RoundsFn) -> Self {
        SHA256Context {
            state: INIT_HASH_VALUES,
            data_len: 0,
            rounds_fn,
        }
    }

    pub fn hash_bytes(&mut self, data: &[u8]) -> [u8; HASH_SIZE] {
        let mut end_chunk = [0_u8; CHUNK_SIZE];

        let mut w = [0_u32; 64];

        for chunk in data.chunks(CHUNK_SIZE) {
            self.data_len += chunk.len() * 8;
            if chunk.len() == CHUNK_SIZE {
                self.process_chunk(&chunk, &mut w);
            } else {
                assert!(chunk.len() < CHUNK_SIZE);
                end_chunk[0..chunk.len()].copy_from_slice(chunk);
            }
        }

        self.finalize(end_chunk, w)
    }

    pub fn hash_file(&mut self, file: File) -> [u8; HASH_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        let mut reader = BufReader::with_capacity(CHUNK_SIZE * 1024, file);

        let mut w = [0_u32; 64];

        while let Ok(bytes_read) = reader.read(&mut chunk[0..CHUNK_SIZE]) {
            self.data_len += bytes_read * 8;
            if bytes_read == CHUNK_SIZE {
                self.process_chunk(&chunk, &mut w);
            } else {
                for i in bytes_read..CHUNK_SIZE {
                    chunk[i] = 0_u8;
                }
                break;
            }
        }

        self.finalize(chunk, w)
    }

    pub fn process_chunk(&mut self, chunk: &[u8], w: &mut [u32; 64]) {
        // Align the chunk of [u8; CHUNK_SIZE=64] to array of [u32; 16].
        let mut bytes = [0_u8; 4];
        assert!(chunk.len() == 64);
        for i in 0..16 {
            bytes[0] = chunk[i * 4 + 0];
            bytes[1] = chunk[i * 4 + 1];
            bytes[2] = chunk[i * 4 + 2];
            bytes[3] = chunk[i * 4 + 3];
            w[i] = u32::from_ne_bytes(bytes).to_be();
        }

        // Fill the rest of the working array using the copied chunk.
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            let new = Wrap(w[i - 16]) + Wrap(s0) + Wrap(w[i - 7]) + Wrap(s1);
            w[i] = new.0;
        }

        // Execute main loop (64 rounds)
        let mut temp = self.state;
        (self.rounds_fn)(&mut temp, w);

        for i in 0..8 {
            self.state[i] = (Wrap(self.state[i]) + Wrap(temp[i])).0;
        }
    }

    pub fn finalize(&mut self, mut chunk: [u8; CHUNK_SIZE], mut w: [u32; 64]) -> [u8; HASH_SIZE] {
        let end_chunk_len = (self.data_len / 8) % CHUNK_SIZE;
        chunk[end_chunk_len] = 0x80;

        // After processing the chunks, the message must be padded with a single '1' bit, followed
        // by K '0' bits and the message length L, such that (L + 1 + K + 64) % 256 == 0 is true.
        if end_chunk_len < CHUNK_MINUS_U64 {
            // In this case, the padding includes: a single '1' bit,  K '0' bits and
            // the message length L (represented as a big-endian 64-bit unsigned int).
            chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            self.process_chunk(&chunk, &mut w);
        } else {
            // Here, the padding includes: a single '1' bit and the first half of '0' bits.
            self.process_chunk(&chunk, &mut w);
            // Then process a new chunk, which consists entirely of padding - the second half of
            // '0' bits and the message length L (represented as a big-endian 64-bit unsigned int).
            chunk = [0_u8; CHUNK_SIZE];
            chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            self.process_chunk(&chunk, &mut w);
        }

        // Convert the state vector values from big-endian representation.
        for i in 0..8 {
            self.state[i] = u32::from_be(self.state[i]);
        }

        // Align the state vector of [u32; 8] to return type of [u8; HASH_SIZE=32].
        unsafe { std::mem::transmute::<[u32; 8], [u8; HASH_SIZE]>(self.state) }
    }
}
