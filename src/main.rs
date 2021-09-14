use std::env::args;
use std::fmt::Write;
use std::fs::File;
use std::time::Instant;

use rayon::iter::*;

mod sha256_impl;
use crate::sha256_impl::*;

fn main() {
    let file_names: Vec<String> = args().skip(1).collect();

    file_names
        .par_iter()
        .map(|file_name| {
            let file = match File::open(file_name) {
                Ok(file) => file,
                Err(e) => return Err(e),
            };

            let mut ctx = SHA256Context::new(sha256_rounds_rust);
            let hash = ctx.hash_file(file);

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
