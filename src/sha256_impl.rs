use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::mem::size_of;
use std::num::Wrapping as Wrap;

extern "C" {
    pub fn sha256_rounds_asm(temp: *mut u32, w: *const u32);
    pub fn sha256_rounds_rust(temp: *mut u32, w: *const u32);
}

// The following initialization data was taken from:
// https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11 and 15)

const INIT_HASH_VALUES: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const CHUNK_SIZE: usize = 512 / 8;
const CHUNK_MINUS_U64: usize = CHUNK_SIZE - size_of::<u64>();
const HASH_SIZE: usize = 256 / 8;

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
    let hash1 = ctx.hash_bytes(msg.as_bytes());
    assert!(hash1 == hash);

    let mut ctx = SHA256Context::new(sha256_rounds_asm);
    let hash2 = ctx.hash_bytes(msg.as_bytes());
    println!("{:x?} {:x?} {:x?}", hash, hash1, hash2);
    assert!(hash2 == hash);
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
}

type RoundsFn = unsafe extern "C" fn(*mut u32, *const u32);

#[repr(C)]
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

        for chunk in data.chunks(CHUNK_SIZE) {
            self.data_len += chunk.len() * 8;
            if chunk.len() == CHUNK_SIZE {
                self.process_chunk(&chunk);
            } else {
                assert!(chunk.len() < CHUNK_SIZE);
                end_chunk[0..chunk.len()].copy_from_slice(chunk);
            }
        }

        self.finalize(end_chunk)
    }

    pub fn hash_file(&mut self, file: File) -> [u8; HASH_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        let mut reader = BufReader::with_capacity(CHUNK_SIZE * 1024, file);

        while let Ok(bytes_read) = reader.read(&mut chunk[0..CHUNK_SIZE]) {
            self.data_len += bytes_read * 8;
            if bytes_read == CHUNK_SIZE {
                self.process_chunk(&chunk);
            } else {
                for i in bytes_read..CHUNK_SIZE {
                    chunk[i] = 0_u8;
                }
                break;
            }
        }

        self.finalize(chunk)
    }

    #[inline(always)]
    pub fn process_chunk(&mut self, chunk: &[u8]) {
        let mut w = [0_u32; 64];

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

        let mut temp = self.state;

        // TODO: Add comment
        unsafe {
            (self.rounds_fn)((&mut temp).as_mut_ptr(), (&w).as_ptr());
        }

        for i in 0..8 {
            self.state[i] = (Wrap(self.state[i]) + Wrap(temp[i])).0;
        }
    }

    pub fn finalize(&mut self, mut end_chunk: [u8; CHUNK_SIZE]) -> [u8; HASH_SIZE] {
        let end_chunk_len = (self.data_len / 8) % CHUNK_SIZE;
        end_chunk[end_chunk_len] = 0x80;

        // After processing the chunks, the message must be padded with a single '1' bit, followed
        // by K '0' bits and the message length L, such that (L + 1 + K + 64) % 256 == 0 is true.
        if end_chunk_len < CHUNK_MINUS_U64 {
            // In this case, the padding includes: a single '1' bit,  K '0' bits and
            // the message length L (represented as a big-endian 64-bit unsigned int).
            end_chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            self.process_chunk(&end_chunk);
        } else {
            // Here, the padding includes: a single '1' bit and the first half of '0' bits.
            self.process_chunk(&end_chunk);
            // Then process a new chunk, which consists entirely of padding - the second half of
            // '0' bits and the message length L (represented as a big-endian 64-bit unsigned int).
            end_chunk = [0_u8; CHUNK_SIZE];
            end_chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            self.process_chunk(&end_chunk);
        }

        // Convert the state vector values from big-endian representation.
        for i in 0..8 {
            self.state[i] = u32::from_be(self.state[i]);
        }

        // Align the state vector of [u32; 8] to return type of [u8; HASH_SIZE=32].
        unsafe { std::mem::transmute::<[u32; 8], [u8; HASH_SIZE]>(self.state) }
    }
}
