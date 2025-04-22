use std::collections::HashSet;
use std::process::Command;
use notify_rust::Notification;

// Keep track of already notified devices (once per session)
static mut ALREADY_NOTIFIED: Option<HashSet<String>> = None;

pub fn create_whitelist_from_connected_devices() -> HashSet<String> {
    let mut whitelist = HashSet::new();

    if let Ok(output) = Command::new("lsusb").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let unique_id = line.trim().to_string(); // Use full line to be unique

                // Clean up device description (more readable)
                let device_name = line
                    .split("ID")
                    .nth(1)
                    .map(|s| s.split_whitespace().skip(1).collect::<Vec<_>>().join(" "))
                    .unwrap_or_else(|| "Unknown Device".into());

                if !device_name.is_empty() {
                    whitelist.insert(device_name.clone());

                    // Notify only if not already notified
                    unsafe {
                        let notified = ALREADY_NOTIFIED.get_or_insert(HashSet::new());
                        if !notified.contains(&unique_id) {
                            Notification::new()
                                .summary("Device Whitelisted")
                                .body(&format!("Whitelisted device:\n{}", device_name))
                                .icon("dialog-information")
                                .show()
                                .unwrap();

                            notified.insert(unique_id);
                        }
                    }
                }
            }
        }
    }

    whitelist
}

pub fn is_device_whitelisted(device_name: &str, whitelist: &HashSet<String>) -> bool {
    whitelist.contains(device_name)
}
