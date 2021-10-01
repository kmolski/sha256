# sha256

My project for the 5th semester Assembly Languages course - an implementation of the SHA256 hashing algorithm
in Rust and AArch64 + x86_64 assembly. Originally, the x86_64 assembly implementation used AVX2 instructions, with
AArch64 and scalar x86_64 (using both [BMI2](https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set)
& standard instructions) implementations added later.

Build & Run:
-----------

To build and run using the assembly implementation:
```sh
cargo run --release -- --impl=asm file1 file2 file3
```
or the Rust implementation:
```sh
cargo run --release -- --impl=rust file1 file2 file3
```

Performance:
-----------

Run time for a 1GiB random dataset (in seconds):

| Intel Core i5-5300U (x86_64) | 1T         | 4T          | 16T   |
| ---------------------------- | ---------- | ----------- | ----- |
| GNU coreutils (1T only)      |      37.39 |         N/A |   N/A |
| Rust                         |      69.34 |       38.46 | 38.13 |
| Assembly (AVX2)              | **139.04** |       53.75 | 53.58 |
| Assembly (BMI2)              |      66.28 | ***32.13*** | 32.22 |
| Assembly (no extensions)     |      70.48 |       36.42 | 36.22 |

| Broadcom BCM2711 (AArch64) | 1T         | 4T          | 16T   |
| -------------------------- | ---------- | ----------- | ----- |
| GNU coreutils (1T only)    |     104.57 |         N/A |   N/A |
| Rust                       |     124.17 | ***34.77*** | 40.53 |
| Assembly                   | **136.74** |       39.60 | 42.23 |

Dependencies:
-----------

- rayon - for parallel processing
- clap - for argument parsing

License:
--------

[MIT License](https://opensource.org/licenses/MIT)