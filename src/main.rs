mod engine;
mod CLI;

use std::{path::Path};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use reqwest;
use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread::JoinHandle;
use std::collections::HashMap;
use lazy_static::lazy_static;
use chrono::Local;
use notify_rust::Notification;


use engine::find_device::find_all_devices;
use CLI::cli::run_cli;
use crate::CLI::whitelist::create_whitelist_from_connected_devices;
use crate::CLI::filehash::{hash_all_files_in_dir, load_hashes_from_file};

// Global log store
lazy_static! {
    pub static ref LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref HASH_SET: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    pub static ref WHITELIST: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    pub static ref FIND_THREAD_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref WHITELIST_READY: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

}

/// Struct to monitor devices in LockDown mode
pub struct DeviceMonitor {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}


/// Downloads SHA256 hashes from Abuse.ch and writes to a local file.
pub fn download_and_save_hashes(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://bazaar.abuse.ch/export/txt/sha256/full/";
    let response = reqwest::blocking::get(url)?.text()?;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for line in response.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

impl DeviceMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start monitoring in a new thread
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        self.handle = Some(thread::spawn(move || {
            if let Err(e) = find_all_devices(running) {
                eprintln!("Find device error: {:?}", e);
            }
        }));
    }

    /// Stop the monitoring thread
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                push_log(format!("Failed to join find_device thread: {:?}", e));
            }
        }
    }
}

/// Run a file hash check in a separate thread
pub fn start_hash_checker(dir: &Path) {
    let path = dir.to_path_buf();
    let hash_thread = thread::spawn(move || {
        let hash_set = HASH_SET.lock().unwrap();
        let hash_set_ref: &HashSet<String> = &hash_set;
        let _ = hash_all_files_in_dir(&path, hash_set_ref);
    });
    // Optionally: hash_thread.join().unwrap();
}

/// Push a new log entry with a timestamp
pub fn push_log(msg: String) {
    let mut logs = LOGS.lock().unwrap();
    let timestamp = Local::now().format("[%H:%M:%S]").to_string();
    logs.push(format!("{} {}", timestamp, msg));
}

/// Retrieve and clear current logs
pub fn get_logs() -> Vec<String> {
    let mut logs = LOGS.lock().unwrap();
    let current_logs = logs.clone();
    logs.clear();
    current_logs
}

fn main() {
    // Start the CLI interface in a separate thread
    let cli_thread = thread::spawn(|| {
        if let Err(e) = run_cli() {
            eprintln!("CLI error: {:?}", e);
        }
    });

    // Load file hashes from project root
    let user_mount_path = format!("{}/hashes.txt", env!("CARGO_MANIFEST_DIR"));
    
    match download_and_save_hashes(&user_mount_path) {
        Ok(_) => push_log("[INFO] Successfully downloaded and saved hashes from Abuse.ch".to_string()),
        Err(e) => push_log(format!("[ERROR] Failed to download hashes: {}", e)),
    }
    
    {
        let mut hash_set = HASH_SET.lock().unwrap();
        *hash_set = load_hashes_from_file(&user_mount_path);
        push_log(format!("[INFO] Loaded {} hashes into memory", hash_set.len()));
    }

    // Initialize whitelist once
    {
        let whitelist_set: HashMap<String, String> = create_whitelist_from_connected_devices();
        let mut whitelist = WHITELIST.write().unwrap();
        *whitelist = whitelist_set;
        WHITELIST_READY.store(true, Ordering::SeqCst);
    }

    // Wait for CLI thread to finish
    let _ = cli_thread.join();
}
