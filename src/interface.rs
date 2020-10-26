type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use glib::clone;
use gtk::prelude::*;

use crate::hash_files;

struct Application {
    builder: gtk::Builder,
    main_window: gtk::ApplicationWindow,
    file_view: gtk::TreeView,
    file_list: gtk::ListStore,
    add_files_btn: gtk::Button,
    remove_files_btn: gtk::Button,
    start_btn: gtk::Button,
    stop_btn: gtk::Button,
    save_results_btn: gtk::Button,
    thread_pool: rayon::ThreadPool,
    thread_pool_stop: AtomicBool,
}

impl Application {}

pub fn init_main_window(builder: &gtk::Builder) -> Result<gtk::ApplicationWindow> {
    let main_window: gtk::ApplicationWindow = builder
        .get_object("main_window")
        .ok_or("main_window not found")?;

    main_window.connect_destroy(|_| gtk::main_quit());
    main_window.show_all();

    Ok(main_window)
}

pub fn exists_in_file_list(new: &PathBuf, file_list: &gtk::ListStore) -> bool {
    let new_name = new.to_str().map(|s| s.to_owned());
    let tree_iter = file_list.get_iter_first();
    loop {
        match &tree_iter {
            Some(iter) => {
                let file_name = file_list.get_value(iter, 0).get::<String>().unwrap();
                if file_name == new_name {
                    break true;
                } else if !file_list.iter_next(iter) {
                    break false;
                }
            }
            _ => {
                break false;
            }
        }
    }
}

pub fn init_add_files_dialog(
    builder: &gtk::Builder,
    main_window: &gtk::ApplicationWindow,
    file_list: &gtk::ListStore,
) -> Result<gtk::Button> {
    let add_files_btn: gtk::Button = builder
        .get_object("add_files_btn")
        .ok_or("add_files_btn not found")?;

    add_files_btn.connect_clicked(clone!(@strong main_window, @strong file_list => move |_| {
        let add_files_dialog = gtk::FileChooserDialog::with_buttons(
            Some("Dodaj pliki"),
            Some(&main_window),
            gtk::FileChooserAction::Open,
            &[
                ("Anuluj", gtk::ResponseType::Cancel),
                ("Dodaj", gtk::ResponseType::Accept),
            ],
        );
        add_files_dialog.set_select_multiple(true);

        if let gtk::ResponseType::Accept = add_files_dialog.run() {
            for file_name in add_files_dialog.get_filenames() {
                if !exists_in_file_list(&file_name, &file_list) {
                    file_list.insert_with_values(None, &[0], &[&file_name.to_str()]);
                }
            }
        }

        add_files_dialog.close();
    }));

    Ok(add_files_btn)
}

pub fn init_remove_files_btn(
    builder: &gtk::Builder,
    file_view: &gtk::TreeView,
    file_list: &gtk::ListStore,
) -> Result<gtk::Button> {
    let remove_files_btn: gtk::Button = builder
        .get_object("remove_files_btn")
        .ok_or("remove_files_btn not found")?;
    remove_files_btn.set_sensitive(false);

    let selection = file_view.get_selection();
    selection.connect_changed(clone!(@strong remove_files_btn => move |selection| {
        let (rows, _) = selection.get_selected_rows();
        remove_files_btn.set_sensitive(rows.len() > 0);
    }));

    remove_files_btn.connect_clicked(clone!(@strong file_view, @strong file_list => move |_| {
        let (rows, _) = selection.get_selected_rows();
        for row_path in rows.into_iter().rev() {
            if let Some(iter) = file_list.get_iter(&row_path) {
                file_list.remove(&iter);
            }
        }
    }));

    Ok(remove_files_btn)
}

pub fn init_start_btn(
    builder: &gtk::Builder,
    use_asm_btn: &gtk::RadioButton,
    thread_count_btn: &gtk::SpinButton,
    progress_bar: &gtk::ProgressBar,
    file_list: &gtk::ListStore,
) -> Result<gtk::Button> {
    let start_btn: gtk::Button = builder
        .get_object("start_btn")
        .ok_or("start_btn not found")?;
    // start_btn.set_sensitive(false);

    start_btn.connect_clicked(clone!(@strong file_list, @strong use_asm_btn,
                                     @strong thread_count_btn, @strong progress_bar => move |_| {
        let mut index_vector = Vec::new();

        let tree_iter = file_list.get_iter_first();
        loop {
            match &tree_iter {
                Some(iter) => {
                    if let Some(file_name) = file_list.get_value(iter, 0).get::<String>().unwrap() {
                        let path = file_list.get_path(&iter).unwrap();
                        index_vector.push((file_name, path.get_indices()[0]));
                    }

                    if !file_list.iter_next(iter) { break; }
                }
                _ => { break; }
            }
        }

        let file_count = index_vector.len() as f64;
        progress_bar.set_fraction(0.0);
        progress_bar.set_text(Some("Obliczanie..."));

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let num_threads = thread_count_btn.get_value_as_int() as usize;
        hash_files(index_vector, tx, use_asm_btn.get_active(), num_threads);

        rx.attach(None, clone!(@strong file_list, @strong progress_bar => move |hash_result| {
            match hash_result {
                Ok((hash_str, index, time)) => {
                    let path = gtk::TreePath::from_indicesv(&[index]);
                    let iter = file_list.get_iter(&path).unwrap();

                    file_list.set(&iter, &[1, 2], &[&hash_str, &time.to_string()]);
                },
                Err((e, index)) => {
                    let path = gtk::TreePath::from_indicesv(&[index]);
                    let iter = file_list.get_iter(&path).unwrap();

                    file_list.set(&iter, &[1, 2], &[&e.as_str(), &"-".to_owned()]);
                }
            }

            let prev_count = (progress_bar.get_fraction() * file_count).ceil().min(file_count);
            progress_bar.set_fraction((prev_count + 1.0) / file_count);
            if prev_count + 1.0 == file_count {
                progress_bar.set_text(Some("Ukończono."));
            }

            glib::Continue(true)
        }));
    }));

    Ok(start_btn)
}

pub fn init_save_results_dialog(
    builder: &gtk::Builder,
    main_window: &gtk::ApplicationWindow,
    file_list: &gtk::ListStore,
) -> Result<gtk::Button> {
    let save_results_btn: gtk::Button = builder
        .get_object("save_results_btn")
        .ok_or("save_results_btn not found")?;

    save_results_btn.connect_clicked(clone!(@strong main_window, @strong file_list => move |_| {
        let save_results_dialog = gtk::FileChooserDialog::with_buttons(
            Some("Zapisz plik wynikowy"),
            Some(&main_window),
            gtk::FileChooserAction::Save,
            &[
                ("Anuluj", gtk::ResponseType::Cancel),
                ("Zapisz", gtk::ResponseType::Accept),
            ],
        );

        save_results_dialog.set_do_overwrite_confirmation(true);

        // TODO: Add error handling dialog here!
        if let gtk::ResponseType::Accept = save_results_dialog.run() {
            if let Some(path) = save_results_dialog.get_filename() {
                let mut output_file = File::create(path).unwrap();

                let tree_iter = file_list.get_iter_first();
                loop {
                    match &tree_iter {
                        Some(iter) => {
                            let file_name = file_list.get_value(iter, 0).get::<String>().unwrap().unwrap();
                            let checksum = file_list.get_value(iter, 1).get::<String>().unwrap();
                            let time = file_list.get_value(iter, 2).get::<String>().unwrap();

                            writeln!(output_file, "#{};{}", time.unwrap(), file_name);
                            writeln!(output_file, "{}  {}", checksum.unwrap(), file_name);

                            if !file_list.iter_next(iter) { break; }
                        }
                        _ => { break; }
                    }
                }
            }
        }

        save_results_dialog.close();
    }));

    Ok(save_results_btn)
}

pub fn init_thread_count_btn(builder: &gtk::Builder) -> Result<gtk::SpinButton> {
    let thread_count_btn: gtk::SpinButton = builder
        .get_object("thread_count_btn")
        .ok_or("thread_count_btn not found")?;

    thread_count_btn.set_value(rayon::current_num_threads() as f64);
    Ok(thread_count_btn)
}
