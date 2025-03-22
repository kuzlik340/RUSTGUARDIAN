```
                       *                                     +                        .                   
                         o                                                                .               
                         .               +                     +  '          |                .*          
   .                                                                        -+-                           
                                                   '                     '   |                         +  
      '       '                                                '                                          
.                     +         '       * '                            +                               .  
                ______          _   _____                     _ _             
                | ___ \        | | |  __ \                   | (_)            
                | |_/ /   _ ___| |_| |  \/_   _  __ _ _ __ __| |_  __ _ _ __  
                |    / | | / __| __| | __| | | |/ _` | '__/ _` | |/ _` | '_ \ 
                | |\ \ |_| \__ \ |_| |_\ \ |_| | (_| | | | (_| | | (_| | | | |
                \_| \_\__,_|___/\__|\____/\__,_|\__,_|_|  \__,_|_|\__,_|_| |_|    
              +                 '                 .         *                                 .           
                         o           .            o                                              |        
     o        .             +                                           '       .               -+-       
               o                                                   .             |          * '  |        
                                                       . '  .                  .-+-                .      
                                               +  .        +                     |          .             
.                   *          ++                                                                                                      
```



## Project Team
- Aliaksei Zimnitski – xzimnitski@stuba.sk
- Timofei Kuzin – xkuzin@stuba.sk

## Introduction
The goal of this project is to develop a **lightweight USB antivirus** with a **CLI** that monitors all **USB devices** in **user space** and blocks them if suspicious activity is detected. In addition to simply blocking unknown devices, the application also **scans files** on newly connected USB flash drives to detect potential malware. This feature is aimed at preventing attacks where a USB drive might deliver malicious executables or scripts.

The project consists of two main parts:

1. **Application Core (User-Space Daemon)** – A Rust-based program that:
    - Continuously **listens for new USB devices** using `udev`.
    - Whenever a device is plugged in, checks if it is in the **Safe Devices List**.
    - If it is a **USB flash drive (mass storage)**, automatically scans its **files** for known malicious signatures (through an external scanning API or local engine like ClamAV).
    - If malware is detected or the device is not allowed, the daemon blocks it and notifies the user.

2. **CLI Interface** – A command-line interface that provides the user with controls to:
    - Add or remove devices from the safe list.
    - View and manage scan results and any suspicious findings.
    - Receive notifications if a newly connected USB device appears malicious or is not approved.
    - Access a device event log (timestamps, device details, scan status, etc.).

By focusing on user-space handling of devices (rather than a kernel module) and integrating a **file scanning step**, we ensure that suspicious USB devices not only get blocked on the basis of unknown hardware IDs but also based on the detection of potentially harmful files on the drive itself.

Requirements
Since the application now operates completely in user space, the main requirements are:

1. Safe Devices Management
    - Ability to add or remove devices (identified by Vendor ID, Product ID, and optionally a serial number) from a “safe list.”
    - Automatic blocking of devices not in the safe list (depending on the enabled mode).

2. Two Monitoring Modes
    - LockDown Mode: Continuously polls all connected USB ports/devices at a chosen interval. If any device doesn’t match the safe list, it gets blocked or triggers a user notification.
    - SafeConnection Mode: Listens for new USB device events from the system (via udev). Whenever a new device is plugged in, the application checks if it’s safe and either allows or blocks it.

3. User Notification
    - Provides real-time alerts (desktop notifications) when a device is blocked, or suspicious behavior is detected.

4. Logging and CLI
    - Logs device connections/disconnections with timestamps and device details.
    - An interactive CLI for:
        - Listing logs
        - Showing and editing the safe list
        - Toggling modes (LockDown or SafeConnection)

Dependencies
User-Space Monitoring (udev)
To monitor USB devices without writing a kernel module, we use udev:
sudo apt-get update
sudo apt-get install libudev-dev
Then, in Cargo.toml:
[dependencies]
udev = "0.6"

Desktop Notifications
For desktop notifications on Linux:
sudo apt-get install libdbus-1-dev
And add to Cargo.toml:
[dependencies]
notify-rust = "4.8"

Terminal Interaction
We use crossterm for CLI input and output:
[dependencies]
crossterm = "0.27"

Logging & Extended Features
- env_logger or any Rust logging framework for structured logs.
- rusqlite (if storing logs or safe device lists in an SQLite database).
- serde + serde_json (if serializing logs or device configurations to JSON).

Below is an example Cargo.toml snippet (combining everything):
[package]
name = "usb_antivirus"
version = "0.1.0"
edition = "2021"

[dependencies]
crossterm = "0.27"
notify-rust = "4.8"
udev = "0.6"
log = "0.4"
env_logger = "0.9"
rusqlite = "0.29"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

Command Examples
A typical CLI might support commands like:
> add device [device_id]  # Add device to the safe list
> del device [device_id]  # Remove device from the safe list
> enable LockDown         # Periodically polls all USB devices (strict mode)
> disable LockDown        # Disables the polling mechanism
> enable SafeConnection   # Reacts only to new devices via udev
> disable SafeConnection  # Stops reacting to new device events
> list logs               # Displays recent device events/logs

# Project diagram

```
                                  ┌──────────────────────────┐          
                                  │                          │          
                                  │  Notification mechanism  │          
                                  │                          │          
                                  └────────────▲─────────────┘          
                                               │                        
                                   ┌───────────┼────────────┐           
                                   │                        │           
                                   │      CLI interface     │           
                                   │                        │           
                                   └─────┬──────────────▲───┘           
                                         │              │               
                              ┌──────────▼──────────────┼──────────────┐
                              │             Kernel module              │
                              │                                        │
                              │  -Polling devices(Lockdown mode)       │
                              │  -Checking new connections(Safe mode)  │
                              │                                        │
                              └────────────────────────────────────────┘
```
