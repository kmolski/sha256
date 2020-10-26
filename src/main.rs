use glib::clone;
use gtk::prelude::*;

mod sha256_impl;
use crate::sha256_impl::*;

mod interface;
use crate::interface::*;

type HashResult = std::result::Result<(String, i32, u64), (String, i32)>;

fn hash_files(
    entries: Vec<(String, i32)>,
    tx: glib::Sender<HashResult>,
    use_asm: bool,
    num_threads: usize,
) {
    use std::fmt::Write;
    use std::fs::File;
    use std::time::Instant;

    fn get_hash_result(file_name: String, index: i32, use_asm: bool) -> HashResult {
        let sha256_impl = if use_asm {
            sha256_rounds_asm
        } else {
            sha256_rounds_rust
        };

        let file = File::open(&file_name).map_err(|err| (err.to_string(), index))?;
        let mut ctx = SHA256Context::new(sha256_impl);

        let start = Instant::now();
        let hash = ctx.hash_file(file);
        let time = start.elapsed().as_millis() as u64;

        let mut hash_str = String::new();

        for byte in &hash {
            write!(hash_str, "{:02x}", byte).map_err(|err| (err.to_string(), index))?;
        }

        Ok((hash_str, index, time))
    }

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to build thread pool");

    for (file_name, index) in entries {
        thread_pool.spawn(clone!(@strong tx => move || {
            tx.send(get_hash_result(file_name, index, use_asm)).unwrap();
        }));
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    gtk::init()?;

    let glade_src = include_str!("./interface.ui");
    let builder = gtk::Builder::from_string(glade_src);

    let main_window = init_main_window(&builder)?;

    let file_list: gtk::ListStore = builder
        .get_object("file_list")
        .ok_or("file_list not found")?;

    let file_view: gtk::TreeView = builder
        .get_object("file_view")
        .ok_or("file_view not found")?;

    let use_asm_btn: gtk::RadioButton = builder
        .get_object("use_asm_btn")
        .ok_or("use_asm_btn not found")?;

    let thread_count_btn = init_thread_count_btn(&builder)?;
    let progress_bar: gtk::ProgressBar = builder
        .get_object("progress_bar")
        .ok_or("progress_bar not found")?;

    // let stop_signal = AtomicBool::new(false);

    let add_files_btn = init_add_files_dialog(&builder, &main_window, &file_list)?;
    let remove_files_btn = init_remove_files_btn(&builder, &file_view, &file_list)?;
    let start_btn = init_start_btn(
        &builder,
        &use_asm_btn,
        &thread_count_btn,
        &progress_bar,
        &file_list,
    )?;
    // let stop_btn = init_stop_btn(&builder, &stop_signal)?;
    let save_results_btn = init_save_results_dialog(&builder, &main_window, &file_list)?;

    gtk::main();
    Ok(())
}
