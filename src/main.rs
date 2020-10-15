use std::env::args;
use std::fs::File;
use std::mem::size_of;
use std::num::Wrapping as Wrap;

use memmap::Mmap;
use rayon::prelude::*;

// The following initialization data was taken from:
// https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf (page 11 and 15)

const INIT_HASH_VALUES: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

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

    let mut ctx = SHA256Context::new(msg.as_bytes());
    assert!(ctx.hash() == hash);
}

#[test]
fn test_string_hash_2() {
    let msg = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let hash = [
        0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E, 0x60,
        0x39, 0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4, 0x19, 0xDB,
        0x06, 0xC1,
    ];

    let mut ctx = SHA256Context::new(msg.as_bytes());
    assert!(ctx.hash() == hash);
}

#[test]
fn test_string_hash_3() {
    let msg = "a".repeat(1_000_000);
    let hash = [
        0xCD, 0xC7, 0x6E, 0x5C, 0x99, 0x14, 0xFB, 0x92, 0x81, 0xA1, 0xC7, 0xE2, 0x84, 0xD7, 0x3E,
        0x67, 0xF1, 0x80, 0x9A, 0x48, 0xA4, 0x97, 0x20, 0x0E, 0x04, 0x6D, 0x39, 0xCC, 0xC7, 0x11,
        0x2C, 0xD0,
    ];

    let mut ctx = SHA256Context::new(msg.as_bytes());
    assert!(ctx.hash() == hash);
}

#[test]
fn test_string_hash_4() {
    let msg = "";
    let hash = [
        0xE3, 0xB0, 0xC4, 0x42, 0x98, 0xFC, 0x1C, 0x14, 0x9A, 0xFB, 0xF4, 0xC8, 0x99, 0x6F, 0xB9,
        0x24, 0x27, 0xAE, 0x41, 0xE4, 0x64, 0x9B, 0x93, 0x4C, 0xA4, 0x95, 0x99, 0x1B, 0x78, 0x52,
        0xB8, 0x55,
    ];

    let mut ctx = SHA256Context::new(msg.as_bytes());
    assert!(ctx.hash() == hash);
}

#[test]
fn test_string_hash_5() {
    let msg = "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    let hash = [
        0xCF, 0x5B, 0x16, 0xA7, 0x78, 0xAF, 0x83, 0x80, 0x03, 0x6C, 0xE5, 0x9E, 0x7B, 0x04, 0x92,
        0x37, 0x0B, 0x24, 0x9B, 0x11, 0xE8, 0xF0, 0x7A, 0x51, 0xAF, 0xAC, 0x45, 0x03, 0x7A, 0xFE,
        0xE9, 0xD1,
    ];

    let mut ctx = SHA256Context::new(msg.as_bytes());
    assert!(ctx.hash() == hash);
}

pub struct SHA256Context<'a> {
    state: [u32; 8], // State vector (256 bits)
    data: &'a [u8],
    data_len: usize, // Data length in bits
}

impl<'a> SHA256Context<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        SHA256Context {
            state: INIT_HASH_VALUES,
            data,
            data_len: 0,
        }
    }

    #[inline(never)]
    pub fn hash(&mut self) -> [u8; HASH_SIZE] {
        let mut end_chunk = [0_u8; CHUNK_SIZE];

        for chunk in self.data.chunks(CHUNK_SIZE) {
            self.data_len += chunk.len() * 8;
            if chunk.len() == CHUNK_SIZE {
                sha256_process_chunk(self, &chunk);
            } else {
                assert!(chunk.len() < CHUNK_SIZE);
                end_chunk[0..chunk.len()].copy_from_slice(chunk);
            }
        }

        let end_chunk_len = (self.data_len / 8) % CHUNK_SIZE;
        end_chunk[end_chunk_len] = 0x80;

        // After processing the chunks, the message must be padded with a single '1' bit, followed
        // by K '0' bits and the message length L, such that (L + 1 + K + 64) % 256 == 0 is true.
        if end_chunk_len < CHUNK_MINUS_U64 {
            // In this case, the padding includes: a single '1' bit,  K '0' bits and
            // the message length L (represented as a big-endian 64-bit unsigned int).
            end_chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            sha256_process_chunk(self, &end_chunk);
        } else {
            // Here, the padding includes: a single '1' bit and the first half of '0' bits.
            sha256_process_chunk(self, &end_chunk);
            // Then process a new chunk, which consists entirely of padding - the second half of
            // '0' bits and the message length L (represented as a big-endian 64-bit unsigned int).
            end_chunk = [0; CHUNK_SIZE];
            end_chunk[CHUNK_MINUS_U64..CHUNK_SIZE].copy_from_slice(&self.data_len.to_be_bytes());
            sha256_process_chunk(self, &end_chunk);
        }

        // Convert the state vector values from big-endian representation.
        for i in 0..8 {
            self.state[i] = u32::from_be(self.state[i]);
        }

        // Align the state vector of [u32; 8] to return type of [u8; HASH_SIZE=32].
        let aligned = unsafe { self.state.align_to::<u8>().1 };
        let mut ret = [0_u8; HASH_SIZE];
        ret.copy_from_slice(aligned);
        ret
    }
}

pub fn sha256_process_chunk(ctx: &mut SHA256Context, chunk: &[u8]) {
    let mut w = [0_u32; 64];

    // Align and copy the chunk of [u8; CHUNK_SIZE=64] to array of [u32; 16].
    let aligned = unsafe { chunk.align_to::<u32>().1 };
    assert!(aligned.len() == 16);
    for i in 0..16 {
        w[i] = u32::to_be(aligned[i]);
    }

    // Fill the rest of the working array using the copied chunk.
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        let new = Wrap(w[i - 16]) + Wrap(s0) + Wrap(w[i - 7]) + Wrap(s1);
        w[i] = new.0;
    }

    let mut temp = ctx.state;

    for i in 0..64 {
        let s1 = temp[4].rotate_right(6) ^ temp[4].rotate_right(11) ^ temp[4].rotate_right(25);
        let ch = (temp[4] & temp[5]) ^ (!temp[4] & temp[6]);
        let temp1 = Wrap(temp[7]) + Wrap(s1) + Wrap(ch) + Wrap(ROUND_VALUES[i]) + Wrap(w[i]);
        let s0 = temp[0].rotate_right(2) ^ temp[0].rotate_right(13) ^ temp[0].rotate_right(22);
        let maj = (temp[0] & temp[1]) ^ (temp[0] & temp[2]) ^ (temp[1] & temp[2]);
        let temp2 = Wrap(s0) + Wrap(maj);

        // TODO: this can be rewritten into rotation + assignment
        // temp.rotate_left(1); temp[4] = (Wrap(temp[4]) + temp1).0; temp[0] = (temp1 + temp2).0
        temp[7] = temp[6];
        temp[6] = temp[5];
        temp[5] = temp[4];
        temp[4] = (Wrap(temp[3]) + temp1).0;
        temp[3] = temp[2];
        temp[2] = temp[1];
        temp[1] = temp[0];
        temp[0] = (temp1 + temp2).0;
    }

    for i in 0..8 {
        ctx.state[i] = (Wrap(ctx.state[i]) + Wrap(temp[i])).0;
    }
}

fn main() {
    let file_names: Vec<String> = args().skip(1).collect();

    file_names
        .par_iter()
        .map(|file_name| {
            let file = match File::open(file_name) {
                Ok(file) => file,
                Err(e) => return Err(e),
            };

            let mmap = unsafe { Mmap::map(&file)? };

            let mut ctx = SHA256Context::new(&mmap);
            let hash = ctx.hash();

            let mut hash_str = String::new();
            for byte in hash.iter() {
                hash_str.push_str(format!("{:02x}", byte).as_str());
            }

            Ok((hash_str, file_name))
        })
        .for_each(|result| match result {
            Ok((hash_str, file_name)) => {
                println!("{}  {}", hash_str, file_name);
            }
            Err(e) => {
                eprintln!("ERROR: {}", e);
            }
        });
}
