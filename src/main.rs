mod engine;
mod cli;

use std::{path::Path};
use std::fs::{File};
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
use std::io::{Cursor, Read};
use zip::read::ZipArchive;
use notify_rust::Notification;
use engine::find_device::find_all_devices;
use engine::process_checker::scan_processes;
use cli::cli::run_cli;
use crate::engine::whitelist::{create_whitelist_from_connected_devices, create_media_whitelist, detect_new_media_mount};
use crate::engine::filehash::{hash_all_files_in_dir, load_hashes_from_file};
use crate::engine::process_checker::ProcessScanResult;
use reqwest::blocking::get;


// Global log store
lazy_static! {
    pub static ref LOGS: RwLock<Vec<String>> = RwLock::new(Vec::new());
    pub static ref HASH_SET: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    pub static ref WHITELIST: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    pub static ref FIND_THREAD_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref WHITELIST_READY: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref WHITELIST_PATHS: RwLock<Vec<String>> = RwLock::new(Vec::new());
}

// Struct to handle the threads in LockDown mode
pub struct DeviceMonitor {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    process_checker_handle: Option<JoinHandle<()>>,
    scan_results: Arc<Mutex<Vec<ProcessScanResult>>>,
}

pub fn detect_new_media_mount_main() -> Option<String> {
    detect_new_media_mount()
}
pub fn create_whitelist_from_connected_devices_main() -> HashMap<String, String>{
    create_whitelist_from_connected_devices()
}

/// Downloads SHA256 hashes from Abuse.ch and writes to a local file.
pub fn download_and_save_hashes(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://bazaar.abuse.ch/export/txt/sha256/full/";
    let response = get(url)?.bytes()?;

    let cursor = Cursor::new(response);
    let mut archive = ZipArchive::new(cursor)?;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    if archive.len() > 0 {
        let mut zipped_file = archive.by_index(0)?;
        let mut buffer = String::new();
        zipped_file.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            let line = line.trim();
            if !line.starts_with('#') && !line.is_empty() {
                writeln!(writer, "{}", line)?;
            }
        }
    } else {
        return Err("ZIP archive is empty".into());
    }

    Ok(())
}

// Methods for the DeviceMonitor structure 
impl DeviceMonitor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            process_checker_handle: None,
            scan_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let scan_results = self.scan_results.clone();

        
        self.handle = Some(thread::spawn(move || {
            if let Err(e) = find_all_devices(running.clone()) {
                eprintln!("Find device error: {:?}", e);
            }
        }));


        let process_running = self.running.clone();
        self.process_checker_handle = Some(thread::spawn(move || {
            while process_running.load(Ordering::Relaxed) {
                let results = scan_processes();
                let mut guard = scan_results.lock().unwrap();
                *guard = results;
                
               
                for proc in guard.iter() {
                    if proc.is_suspicious {
                        push_log(format!(
                            "[ALERT] PID: {}, Name: {}, Reason: {}",
                            proc.pid,
                            proc.name,
                            proc.reason.as_ref().unwrap_or(&"Unknown".to_string()))
                        );
                        Notification::new()
                            .summary("Malicious Process Alert")
                            .body(&format!("Registered malicious process {} \n", proc.name))
                            .icon("dialog-warning")
                            .show()
                            .ok(); 
                    }
                }
                push_log("[RESULT] Processes are fine, no malicious were found".to_string());
                let sleep_total = Duration::from_secs(60);
                let sleep_step = Duration::from_secs(1); 
                let mut elapsed = Duration::ZERO;

                while elapsed < sleep_total {
                    if !process_running.load(Ordering::Relaxed) {
                        break;
                    }
                    thread::sleep(sleep_step);
                    elapsed += sleep_step;
                }
            }

        }));
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                push_log(format!("Thread error: {:?}", e));
            }
        }

        if let Some(handle) = self.process_checker_handle.take() {
            if let Err(e) = handle.join() {
                push_log(format!("Thread error: {:?}", e));
            }
        }
    }

    pub fn get_scan_results(&self) -> Vec<ProcessScanResult> {
        self.scan_results.lock().unwrap().clone()
    }
}

// Run a file hash check in a separate thread
// Basically this starts a thread that will hash all files in the directory on the flash drive
// So our program could then compare hashes to the hashes of viruses
pub fn start_hash_checker(dir: &Path) {
    let hash_set_clone = {
        let hash_set = HASH_SET.lock().unwrap();
        if hash_set.is_empty() {
            push_log("There is no file to compare hashes, \
                      please ensure that you have them or enable internet connection on the device so program could download them".to_string());
            return;
        }
        hash_set.clone() 
    };
    let path = dir.to_path_buf();
    let _hash_thread = thread::spawn(move || 
        {
            
            let _ = hash_all_files_in_dir(&path, &hash_set_clone);
            push_log(format!("[INFO] Finished checking {}", path.display()));
        }
    );
}

// Push a new log entry with a timestamp for the TUI CLI interface
pub fn push_log(msg: String) {
    let mut logs = LOGS.write().unwrap();
    let timestamp = Local::now().format("[%H:%M:%S]").to_string();
    logs.push(format!("{} {}", timestamp, msg));
}

// Retrieve and clear current logs for printing
pub fn get_logs() -> Vec<String> {
    let logs = LOGS.read().unwrap();
    let current_logs = logs.clone();
    current_logs
}

pub fn clear_logs() {
    let mut logs = LOGS.write().unwrap();
    logs.clear();
}


fn main() {
    // Start the CLI interface in a separate thread
    let cli_thread = thread::spawn(|| {
        if let Err(e) = run_cli() {
            eprintln!("CLI error: {:?}", e);
        }
    });
    push_log("[INFO] Preparing the RustGuardian for work, please wait".to_string());

    // Load file hashes from project root. This operation takes about 8 seconds
    let device_hashes_path = format!("{}/hashes.txt", env!("CARGO_MANIFEST_DIR"));
    let mut hashes_exists = false;
    let mut update_needed = false;

    if Path::new(&device_hashes_path).exists() {
        push_log("[INFO] Hashes.txt was found on device".to_string());
        hashes_exists = true;
        if let Ok(metadata) = std::fs::metadata(&device_hashes_path) {
            if let Ok(modified_time) = metadata.modified() {
                use std::time::{SystemTime, Duration};
                if let Ok(elapsed) = SystemTime::now().duration_since(modified_time) {
                    if elapsed > Duration::from_secs(48 * 3600) {
                        push_log("[WARNING] Hashes.txt is older than 48 hours, updating...".to_string());
                        update_needed = true;
                    } else {
                        push_log("[INFO] Hashes.txt is fresh".to_string());
                    }
                }
            }
        }
    } 
    if update_needed || !hashes_exists {
        match download_and_save_hashes(&device_hashes_path) {
            Ok(_) => {
                push_log("[INFO] Hashes are downloaded from abuse.ch".to_string());
                hashes_exists = true;
                update_needed = false;
            }
            Err(e) => push_log(format!("[ERROR] Error when trying to load hashes: {}", e)),
        }
    }

    if hashes_exists {
        if update_needed {
            push_log("[WARNING] The local hash database is older than 48 hours. Please connect to the internet to update it. RustGuardian \
            will continue working with the outdated hashes for now.".to_string());
        }
        push_log("[INFO] Extracting hashes, please wait".to_string());
        {
            let mut hash_set = HASH_SET.lock().unwrap();
            *hash_set = load_hashes_from_file(&device_hashes_path);
            // Inserting hash of our non malicous file for testing
            hash_set.insert("195c291a262a846cefa5b42fc8a74293cf91bfe44d49c71ed56a07b588b1ecba".to_string());
            push_log(format!("[INFO] Loaded {} hashes into memory", hash_set.len()));
        }
    }
    else{
        push_log("[ERROR] Hashes could not be extracted. The SafeConnection mode is disabled".to_string());
    }
    push_log("[INFO] Initializing whitelist, please wait".to_string());
    // Initialize whitelist once
    {
        let whitelist_set: HashMap<String, String> = create_whitelist_from_connected_devices();
        let mut whitelist = WHITELIST.write().unwrap();
        *whitelist = whitelist_set;
        let whitelist_media_vector: Vec<String> = create_media_whitelist();
        let mut whitelist_media = WHITELIST_PATHS.write().unwrap();
        *whitelist_media = whitelist_media_vector;
        WHITELIST_READY.store(true, Ordering::SeqCst);
    }

    push_log("The RustGuardian is prepared to guard your connections!".to_string());
    // Wait for CLI thread to finish
    let _ = cli_thread.join();
}
