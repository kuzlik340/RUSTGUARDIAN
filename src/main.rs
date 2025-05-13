mod engine;
mod CLI;
use std::{fs, path::Path};
use std::thread;
use std::sync::{Arc, Mutex};
use engine::find_device::find_all_devices;
use CLI::cli::run_cli;
use lazy_static::lazy_static;
use notify_rust::Notification;
use crate::CLI::whitelist::create_whitelist_from_connected_devices;
use crate::CLI::filehash::{hash_all_files_in_dir, load_hashes_from_file};
use std::collections::HashSet;


lazy_static! {
    pub static ref LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

lazy_static! {
    pub static ref HASH_SET: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
}

pub fn start_hash_checker(dir: &Path) {
    let path = dir.to_path_buf();

    let hash_thread = thread::spawn(move || {
        let hash_set = HASH_SET.lock().unwrap();
        let hash_set_ref: &HashSet<String> = &hash_set;
        
        if let result = hash_all_files_in_dir(&path, hash_set_ref) {
            // Обработка результатов
        }
    });

    //hash_thread.join().unwrap();
}

pub fn start_find_device() {
    let find_thread = thread::spawn(|| {
        //println!("Starting thread");
        if let Err(e) = find_all_devices() {
            eprintln!("Find device error: {:?}", e);
        }
    });
}

pub fn push_log(msg: String) {
    let mut logs = LOGS.lock().unwrap();
    logs.push(msg);
}

pub fn get_logs() -> Vec<String> {
    let mut logs = LOGS.lock().unwrap();
    let current_logs = logs.clone();
    logs.clear();
    current_logs
}

fn main() {
    let cli_thread = thread::spawn(|| {
        if let Err(e) = run_cli() {
            eprintln!("CLI error: {:?}", e);
        }
    });
    {
        let mut hash_set = HASH_SET.lock().unwrap();
        *hash_set = load_hashes_from_file("/home/timkuz/RUST_PROJECT/RUST_PROJECT/hashes.txt");
    }

    let whitelist = create_whitelist_from_connected_devices();


    // А в основном потоке запускаем мониторинг
    //find_all_devices();

    // Ждём завершения CLI, если find_all_devices когда-то завершится
    let _ = cli_thread.join();
}