use udev::{MonitorBuilder, EventType};
use super::{keylogger, whitelist::WHITELIST};
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use crate::push_log;
use std::sync::atomic::{AtomicBool, Ordering};

/* This is the function that will find all new connections of the input devices.
 * This is used in the LockDown mode so we could then intercept what the keyboard is writing into system */
pub fn find_all_devices(running: Arc<AtomicBool>) -> std::io::Result<()> {
    let monitor = MonitorBuilder::new()?.listen()?; // Create a monitor for monitoring events on all input devices
    push_log(format!("[INFO] Starting scanning for input devices, waiting for new devices..."));
    // Variables for controling the child threads
    let keyloggers_running = Arc::new(AtomicBool::new(true));
    let mut keylogger_threads = Vec::new();
    //let mut seen_devnodes = std::collections::HashSet::new();
    // Run while flag running is true (could be stopped from main.rs)
    while running.load(Ordering::Relaxed) {
        // Iterate through all events
        if let Some(event) = monitor.iter().next() {
            if event.event_type() == EventType::Add {
                let device = event.device();
                // Check if the device is a keyboard 
                if let Some(id_input_keyboard) = device.property_value("ID_INPUT_KEYBOARD") {
                    // Check if we found the keyboard
                    if id_input_keyboard.to_str() == Some("1") {
                        let mut name_str: String = String::from("UNKNOWN");
                        // Retrieving the device characteristics
                        if let Some(name) = device.property_value("NAME") {
                            name_str = name.to_string_lossy().into_owned();
                        }
                        name_str = name_str.trim().trim_matches('"').to_string();
                        for k in WHITELIST.read().unwrap().keys() {
                            push_log(format!("WHITELIST KEY: {:?}", k));
                        }
                        // Check if device is whitelisted
                        if WHITELIST.read().unwrap().contains_key(&name_str){
                            continue
                        }
                        let mut dev_identificator: String = String::from("NULL");
                        match device.parent_with_subsystem("usb") {
                            Ok(Some(parent)) => {
                                // We have a parent device with subsystem "usb"
                                let parent_path = parent.sysname();
                                dev_identificator = parent_path.to_string_lossy().into_owned();
                            }
                            Ok(None) => {
                                // Subsystem "usb" not found in the parents
                                push_log(format!("Subsystem USB not found in the parents"));
                            }
                            Err(e) => {
                                // An error occurred reading the parent device
                                push_log(format!("Error looking up USB parent: {}", e));
                            }
                        }

                        if let Some(devnode) = device.devnode() {
                            // Creating a datapath that will contain events for the new keyboard
                            let devnode_str = devnode.to_str().unwrap().to_string();
                            
                            let keyloggers_running_clone = keyloggers_running.clone();
                            // Start logging in a new thread
                            let handle = thread::spawn(move || {
                                push_log("[INFO] Starting logging new events on device".to_string());
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
        thread::sleep(Duration::from_millis(50)); // A bit of relax for processor
    }
    keyloggers_running.store(false, Ordering::Relaxed);
    // Stop all child threads
    for handle in keylogger_threads {
        if let Err(e) = handle.join() {
            push_log(format!("Failed to join keylogger thread: {:?}", e));
        }
    }
    
    push_log("Device monitoring stopped".to_string());
    Ok(())
}