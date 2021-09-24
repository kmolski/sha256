use std::fmt::Write;
use std::fs::File;

use clap::{App, Arg};
use rayon::iter::*;

mod sha256_impl;
use crate::sha256_impl::*;

fn main() {
    let impls_names = SHA256_IMPLS
        .iter()
        .map(|entry| entry.0)
        .collect::<Vec<&str>>();

    let app = App::new("sha256")
        .arg(
            Arg::from_usage("[impl] -i --impl=[impl] 'implementation to use'")
                .possible_values(&impls_names),
        )
        .arg(Arg::from_usage("<INPUT> 'input files to hash'"))
        .version("0.1.0");

    let matches = app.get_matches();

    let impl_name = matches.value_of("impl").unwrap_or("rust");
    let file_names: Vec<&str> = matches
        .values_of("INPUT")
        .expect("No input files specified!")
        .collect();

    let hash_implementation = SHA256_IMPLS
        .binary_search_by(|(k, _)| k.cmp(&impl_name))
        .map(|x| SHA256_IMPLS[x].1)
        .expect("Invalid SHA implementation chosen!");

    file_names
        .par_iter()
        .map(|file_name| {
            let file = File::open(file_name).map_err(|err| err.to_string())?;

            let mut ctx = SHA256Context::new(hash_implementation);
            let hash = ctx.hash_file(file);

            let mut hash_str = String::with_capacity(32 * 2);
            for byte in hash.iter() {
                write!(hash_str, "{:02x}", byte).map_err(|err| err.to_string())?;
            }

            Ok((hash_str, file_name))
        })
        .for_each(|result: Result<_, String>| match result {
            Ok((hash_str, file_name)) => {
                println!("{}  {}", hash_str, file_name);
            }
            Err(e) => {
                eprintln!("ERROR: {}", e);
            }
        });
}
