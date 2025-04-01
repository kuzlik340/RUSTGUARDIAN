
mod engine;

use udev::{MonitorBuilder, EventType};
use engine::{keylogger};
use std::thread;
use std::borrow::Cow;
fn main() -> std::io::Result<()> {
    let mut monitor = MonitorBuilder::new()?.listen()?; // Create a monitor for monitoring events on all input devices
    println!("Starting RustGuardian, waiting for new devices...");

    loop {
        if let Some(event) = monitor.next() {
            if event.event_type() == EventType::Add {
                let device = event.device();
                /* Check if the device is a keyboard */
                if let Some(id_input_keyboard) = device.property_value("ID_INPUT_KEYBOARD") {
                    if id_input_keyboard.to_str() == Some("1") {
                        let mut name_str: Cow<str> = Cow::Borrowed("NULL");
                        if let Some(name) = device.property_value("NAME") {
                            name_str = name.to_string_lossy();
                            println!("Device name: {}", name_str);
                        }
                        if let Some(devnode) = device.devnode() {
                            if devnode.to_str().map(|s| s.contains("/dev/input")).unwrap_or(false) {
                                println!("Main keyboard device event: {}", devnode.display());
                            }


                            let devnode_str = devnode.to_str().unwrap().to_string();
                            let name_str_owned = name_str.into_owned();
                            // Start logging in a new thread
                            thread::spawn(move || {
                                if let Err(e) = keylogger::start_logging(&devnode_str, &name_str_owned) {
                                    eprintln!("Error in keylogger: {}", e);
                                }
                            });

                        }
                    }
                }
            }
        }
    }
}



