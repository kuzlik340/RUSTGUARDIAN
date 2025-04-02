
mod engine;

use udev::{MonitorBuilder, EventType};
use engine::keylogger;
use std::thread;
use std::borrow::Cow;
use std::path::PathBuf;
fn main() -> std::io::Result<()> {
    let mut monitor = MonitorBuilder::new()?.listen()?; // Create a monitor for monitoring events on all input devices
    println!("Starting RustGuardian, waiting for new devices...");

    loop {
        if let Some(event) = monitor.iter().next() {
            if event.event_type() == EventType::Add {
                let device = event.device();
                /* Check if the device is a keyboard */
                if let Some(id_input_keyboard) = device.property_value("ID_INPUT_KEYBOARD") {
                    if id_input_keyboard.to_str() == Some("1") {
                        let mut name_str: String = String::from("NULL");
                        if let Some(name) = device.property_value("NAME") {
                            name_str = name.to_string_lossy().into_owned();
                            println!("Device name: {}", name_str);
                        }
                        let mut dev_identificator: String = String::from("NULL");
                        match device.parent_with_subsystem("usb") {
                            Ok(Some(parent)) => {
                                // We have a parent device with subsystem "usb"
                                let parent_path = parent.sysname();
                                dev_identificator = parent_path.to_string_lossy().into_owned();
                                println!("USB parent syspath: {}", dev_identificator);

                            }
                            Ok(None) => {
                                // Subsystem "usb" not found in the parents
                                println!("No USB parent subsystem found.");
                            }
                            Err(e) => {
                                // An error occurred reading the parent device
                                eprintln!("Error looking up USB parent: {}", e);
                            }
                        }


                        if let Some(devnode) = device.devnode() {
                            if devnode.to_str().map(|s| s.contains("/dev/input")).unwrap_or(false) {
                                println!("Main keyboard device event: {}", devnode.display());
                            }


                            let devnode_str = devnode.to_str().unwrap().to_string();
                            // Start logging in a new thread
                            thread::spawn(move || {
                                if let Err(e) = keylogger::start_logging(&devnode_str, &dev_identificator, &name_str) {
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


