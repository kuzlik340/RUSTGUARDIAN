use udev::{MonitorBuilder, EventType};
use super::keylogger;
use std::thread;
use std::io::{Write};
use std::os::unix::io::AsRawFd;


use std::time::Duration;

use std::process::Command;
use std::thread::sleep;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::push_log;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn find_all_devices(running: Arc<AtomicBool>) -> std::io::Result<()> {
    let mut monitor = MonitorBuilder::new()?.listen()?; // Create a monitor for monitoring events on all input devices
    //println!("Starting RustGuardian, waiting for new devices...");
    push_log(format!("Starting RustGuardian, waiting for new devices..."));
    let keyloggers_running = Arc::new(AtomicBool::new(true));
    let mut keylogger_threads = Vec::new();
    while running.load(Ordering::Relaxed) {
        if let Some(event) = monitor.iter().next() {
            if event.event_type() == EventType::Add {
                let device = event.device();
                /* Check if the device is a keyboard */
                if let Some(id_input_keyboard) = device.property_value("ID_INPUT_KEYBOARD") {
                    if id_input_keyboard.to_str() == Some("1") {
                        let mut name_str: String = String::from("UNKNOWN");
                        if let Some(name) = device.property_value("NAME") {
                            name_str = name.to_string_lossy().into_owned();
                            push_log(format!("Device name: {}", name_str));
                        }
                        let mut dev_identificator: String = String::from("NULL");
                        match device.parent_with_subsystem("usb") {
                            Ok(Some(parent)) => {
                                // We have a parent device with subsystem "usb"
                                let parent_path = parent.sysname();
                                dev_identificator = parent_path.to_string_lossy().into_owned();
                                push_log(format!("USB parent syspath: {}", dev_identificator));

                            }
                            Ok(None) => {
                                // Subsystem "usb" not found in the parents
                                push_log(format!("No USB parent subsystem found."));
                            }
                            Err(e) => {
                                // An error occurred reading the parent device
                                eprintln!("Error looking up USB parent: {}", e);
                            }
                        }

                        if let Some(devnode) = device.devnode() {

                            if devnode.to_str().map(|s| s.contains("/dev/input")).unwrap_or(false) {
                                push_log(format!("Main keyboard device event: {}", devnode.display()));
                            }




                            let devnode_str = devnode.to_str().unwrap().to_string();
                            // Start logging in a new thread

                            let keyloggers_running_clone = keyloggers_running.clone();

                            // передаём в логгер
                            let handle = thread::spawn(move || {
                                if let Err(e) = keylogger::start_logging(&devnode_str, &dev_identificator, &name_str, keyloggers_running_clone) {
                                    eprintln!("Error in keylogger: {}", e);
                                }
                            });
                            keylogger_threads.push(handle);

                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    keyloggers_running.store(false, Ordering::Relaxed);
    
    for handle in keylogger_threads {
        if let Err(e) = handle.join() {
            push_log(format!("Failed to join keylogger thread: {:?}", e));
        }
    }
    
    push_log("Device monitoring stopped".to_string());
    //sleep(Duration::from_millis(30000));
    Ok(())
}