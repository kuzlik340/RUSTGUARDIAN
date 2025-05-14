use evdev::{Device, InputEventKind, Key};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::io::{Write};
use chrono::Local;
use std::sync::Arc;
use crate::push_log;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn start_logging(device_event_path: &str, device_path: &str, device_name: &str, running: Arc<AtomicBool>) -> std::io::Result<()> {
    /* Open device events with the path that will be sent from find_device thread */
    let mut device = Device::open(device_event_path).expect("Failed to open device");
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .mode(0o600)
        .open("logg.txt")?;
    push_log("All events from keyboars will be saved into logg.txt".to_string());
    let now = Local::now(); // Gets local time
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    write!(log_file, "[{}] Starting listening for events on the device with path: {}\n", timestamp, device_event_path)?;
    log_file.flush()?;
    push_log(format!("Starting listening for keyboard activities"));
    let key_map = create_keymap();
    let mut backspace_found: bool = false;
    let mut timestamps: Vec<u128> = Vec::new();
    'outer: while running.load(Ordering::Relaxed) {
        for ev in device.fetch_events().expect("Failed to fetch events") {
            if let InputEventKind::Key(key) = ev.kind() {
                if ev.value() == 1 { //check time difference 
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();
                    /* Saving timestamp of click since we want to measure the difference in time between each clicks */
                    timestamps.push(now);
                    
                    // Check the average speed of clicking for the first 7 clicks
                    if timestamps.len() >= 7 {
                        let mut time_diff_clicks: Vec<u128> = Vec::new();
                        for i in 1..timestamps.len() {
                            time_diff_clicks.push(timestamps[i] - timestamps[i - 1]);
                        }
                        let alltime: u128 = time_diff_clicks.iter().sum();
                        let avg_speed_in_ms = alltime as f32 / time_diff_clicks.len() as f32;
                        let mut too_small_diff = 0;
                        for &diff in &time_diff_clicks {
                            if (diff as f32 - avg_speed_in_ms).abs() < 150.0 {
                                too_small_diff += 1;
                            }
                        }
                        if too_small_diff > 5 {
                            push_log(format!("WARNING RustGuardian registered BadUSB attack, the device will be unmounted"));
                            unmount_device(device_path)?;
                            push_log(format!("Device was succesfully removed"));
                        }
                        else{
                            push_log("The device was scanned, not a BAD USB".to_string());
                        }
                        break 'outer;
                    }
                    if key != Key::KEY_BACKSPACE {
                        backspace_found = false;
                        if let Some(character) = key_map.get(&key) {
                            log_file.write_all(character.as_bytes())?;
                            log_file.flush()?;
                        }
                    } else {
                        if backspace_found {
                            continue;
                        }
                        else{
                            log_file.write_all(b"\n")?;
                            log_file.flush()?;
                            backspace_found = true;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}


fn unmount_device(sysfs_device_path: &str) -> std::io::Result<()> {
    // Datapath example: /sys/bus/usb/devices/2-1/authorized. 
    // When changing option in authorised the device could not then communicate with OS
    let authorized_file = format!("/sys/bus/usb/devices/{}/authorized", sysfs_device_path);

    // Writing 0 to disable device
    let mut file = OpenOptions::new()
        .write(true)
        .open(&authorized_file)?;

    file.write_all(b"0")?;

    push_log(format!("Power off for {} device (authorized=0)", sysfs_device_path));
    Ok(())
}




/* Hashmap to write the text as user inputs it */
fn create_keymap() -> HashMap<Key, &'static str> {
    HashMap::from([
        (Key::KEY_A, "a"), (Key::KEY_B, "b"), (Key::KEY_C, "c"), (Key::KEY_D, "d"),
        (Key::KEY_E, "e"), (Key::KEY_F, "f"), (Key::KEY_G, "g"), (Key::KEY_H, "h"),
        (Key::KEY_I, "i"), (Key::KEY_J, "j"), (Key::KEY_K, "k"), (Key::KEY_L, "l"),
        (Key::KEY_M, "m"), (Key::KEY_N, "n"), (Key::KEY_O, "o"), (Key::KEY_P, "p"),
        (Key::KEY_Q, "q"), (Key::KEY_R, "r"), (Key::KEY_S, "s"), (Key::KEY_T, "t"),
        (Key::KEY_U, "u"), (Key::KEY_V, "v"), (Key::KEY_W, "w"), (Key::KEY_X, "x"),
        (Key::KEY_Y, "y"), (Key::KEY_Z, "z"),
        (Key::KEY_1, "1"), (Key::KEY_2, "2"), (Key::KEY_3, "3"), (Key::KEY_4, "4"),
        (Key::KEY_5, "5"), (Key::KEY_6, "6"), (Key::KEY_7, "7"), (Key::KEY_8, "8"),
        (Key::KEY_9, "9"), (Key::KEY_0, "0"),
        (Key::KEY_SPACE, " "), (Key::KEY_ENTER, "\n"),
        (Key::KEY_MINUS, "-"), (Key::KEY_EQUAL, "="),
        (Key::KEY_LEFTBRACE, "["), (Key::KEY_RIGHTBRACE, "]"),
        (Key::KEY_BACKSLASH, "\\"), (Key::KEY_SEMICOLON, ";"),
        (Key::KEY_APOSTROPHE, "'"), (Key::KEY_GRAVE, "`"),
        (Key::KEY_COMMA, ","), (Key::KEY_DOT, "."), (Key::KEY_SLASH, "/"),
        (Key::KEY_TAB, "\t"), (Key::KEY_BACKSPACE, "\0"),
    ])
}