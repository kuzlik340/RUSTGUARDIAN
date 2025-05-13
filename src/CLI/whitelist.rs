use std::collections::HashSet;
use std::process::Command;
use notify_rust::Notification;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global static variable to track already notified devices within this session.
/// Unsafe is required because mutable static variables can lead to data races.
static ALREADY_NOTIFIED: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Creates a whitelist of currently connected USB devices using `lsusb`.
/// For each new device detected (based on its unique ID), a desktop notification is shown.
/// Returns a HashSet of device names.
pub fn create_whitelist_from_connected_devices() -> HashSet<String> {
    let mut whitelist = HashSet::new();

    // Run `lsusb` command to list connected USB devices
    if let Ok(output) = Command::new("lsusb").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let unique_id = line.trim().to_string(); // Use the full line as a unique device ID

                // Extract device description from the line (after "ID ...")
                let device_name = line
                    .split("ID")
                    .nth(1)
                    .map(|s| s.split_whitespace().skip(1).collect::<Vec<_>>().join(" "))
                    .unwrap_or_else(|| "Unknown Device".into());

                if !device_name.is_empty() {
                    whitelist.insert(device_name.clone());

                    // Show notification only if this device hasn't been notified yet
                    let mut notified = ALREADY_NOTIFIED.lock().unwrap();
                    if !notified.contains(&unique_id) {
                        if let Err(e) = Notification::new()
                            .summary("Device Whitelisted")
                            .body(&format!("Whitelisted device:\n{}", device_name))
                            .icon("dialog-information")
                            .show()
                        {
                            eprintln!("Failed to show notification: {}", e);
                        }
                        notified.insert(unique_id);
                    }
                }
            }
        }
    }

    whitelist
}

