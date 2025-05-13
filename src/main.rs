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
use chrono::Local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread::JoinHandle;

pub struct DeviceMonitor {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DeviceMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        
        self.handle = Some(thread::spawn(move || {
            if let Err(e) = find_all_devices(running) {
                eprintln!("Find device error: {:?}", e);
            }
        }));
    }
    
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                push_log(format!("Failed to join find_device thread: {:?}", e));
            }
        }
    }
}

lazy_static! {
    pub static ref LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

lazy_static! {
    pub static ref HASH_SET: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
}

lazy_static! {
    pub static ref FIND_THREAD_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}


pub fn start_hash_checker(dir: &Path) {
    let path = dir.to_path_buf();
    let hash_thread = thread::spawn(move || {
        let hash_set = HASH_SET.lock().unwrap();
        let hash_set_ref: &HashSet<String> = &hash_set;
        
        if let result = hash_all_files_in_dir(&path, hash_set_ref) {
           
        }
    });

    //hash_thread.join().unwrap();
}

pub fn push_log(msg: String) {
    let mut logs = LOGS.lock().unwrap();
    let timestamp = Local::now().format("[%H:%M:%S]").to_string();
    logs.push(format!("{} {}", timestamp, msg));
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
        let mut user_mount_path = format!("/home/{}/RUST_PROJECT/RUST_PROJECT/hashes.txt", std::env::var("USER").unwrap_or_else(|_| "debian".into()));
        *hash_set = load_hashes_from_file(&user_mount_path);
    }

    let whitelist = create_whitelist_from_connected_devices();


    // А в основном потоке запускаем мониторинг
    //find_all_devices();

    // Ждём завершения CLI, если find_all_devices когда-то завершится
    let _ = cli_thread.join();
}