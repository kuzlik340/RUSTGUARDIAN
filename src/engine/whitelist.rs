use std::process::Command;
use lazy_static::lazy_static;
use crate::RwLock;
use crate::WHITELIST_PATHS;
/// Tracks which device IDs we've already shown notifications for
use std::collections::HashMap;
use crate::Path;
use std::fs;

lazy_static! {
    pub static ref WHITELIST: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

/// Scans currently connected USB devices using `lsusb`,
/// extracts their ID (e.g. 046d:c534), shows notification if not yet shown,
/// and returns a set of all connected device IDs.
pub fn create_whitelist_from_connected_devices() -> HashMap<String, String> {
    let mut devices = HashMap::new();

    if let Ok(output) = Command::new("lsusb").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if let Some(id_part) = line.split("ID").nth(1) {
                    let mut parts = id_part.trim().split_whitespace();
                    let id = parts.next().unwrap_or("unknown").to_string();
                    let name = parts.collect::<Vec<_>>().join(" ");
                    if !id.is_empty() {
                        devices.insert(id, name);
                    }
                }
            }
        }
    }

    devices
}

// Function that will return new path in /media/user (used for checking new flash drives in SafeConnection mode)
pub fn detect_new_media_mount() -> Option<String> {
    let user_mount_path = format!(
        "/media/{}",
        std::env::var("SUDO_USER")
            .unwrap_or_else(|_| std::env::var("USER")
            .expect("Neither SUDO_USER nor USER is set"))
    );

    let media_path = Path::new(&user_mount_path);
    
    if let Ok(entries) = fs::read_dir(media_path) {
        for entry in entries.flatten() {

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let mount_str = path.to_string_lossy().to_string();

            {
                let whitelist_media = WHITELIST_PATHS.read().unwrap();
                if whitelist_media.contains(&mount_str){
                    continue;
                }
            }

            {
                let mut whitelist_media = WHITELIST_PATHS.write().unwrap();
                if !whitelist_media.contains(&mount_str) {
                    whitelist_media.push(mount_str.clone());
                    return Some(mount_str);
                }
            }
        }
    }

    None
}

// Will create whitelist of /media/user paths
pub fn create_media_whitelist() -> Vec<String> {
    let mut safe_paths = Vec::new();
    let user_mount_path = format!(
                "/media/{}",
                std::env::var("SUDO_USER")
                    .unwrap_or_else(|_| std::env::var("USER")
                    .expect("Neither SUDO_USER nor USER is set"))
    );
    let media_path = Path::new(&user_mount_path);
    if let Ok(entries) = fs::read_dir(media_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                safe_paths.push(path.to_string_lossy().to_string());
            }
        }
    }
    safe_paths
}

