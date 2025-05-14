use std::collections::HashSet;
use std::process::Command;
use lazy_static::lazy_static;
use crate::RwLock;

/// Tracks which device IDs we've already shown notifications for
use std::collections::HashMap;

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

