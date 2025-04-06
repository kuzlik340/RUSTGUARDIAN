use std::collections::HashSet;
use std::process::Command;

// Creates a whitelist of allowed device names
pub fn create_whitelist_from_connected_devices() -> HashSet<String> {
    let mut whitelist = HashSet::new();

    if let Ok(output) = Command::new("lsusb").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if let Some(device_name) = line.split("ID").nth(1).and_then(|s| s.split_whitespace().skip(1).collect::<Vec<_>>().join(" ").into()) {
                    if !device_name.trim().is_empty() {
                        whitelist.insert(device_name.trim().to_string());
                    }
                }
            }
        }
    }

    whitelist
}

// Checks if a device is in the whitelist
pub fn is_device_whitelisted(device_name: &str, whitelist: &HashSet<String>) -> bool {
    whitelist.contains(device_name)
}
