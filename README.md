# RUST_PROJECT
## Project Team
- **Aliaksei Zimnitski** – xzimnitski@stiba.sk
- **Timofei Kuzin** – xkuzin@stuba.sk


## Introduction

The goal of this project is to develop a **kernel module** with a **user-friendly interface** that monitors all **USB devices** and blocks them if malicious activity is detected.

The project consists of two main components:

1. **Kernel Module** – This module operates at the **kernel level**, continuously monitoring USB devices to track their activity. For example, it will check whether a **USB flash drive remains a storage device** or has re-enumerated as an **HID device** (such as a keyboard).

2. **User-Space Module** – This component runs in **user space** and provides a **CLI interface** along with **real-time notifications** when a potentially harmful USB device is detected.

Through this project, we want to get experience with Rust’s memory safety features, concurrency, and system-level programming, while building something useful and security-focused! 

## Requirements
The kernel module itself will provide:
1) Two or more modes to work. Lockdown mode which will open scan all devices without any sleep. Also a mode which will just compare if connected device is in the safe list.
2) The user-space API: Adding/Deleting safe devices, start Lockdown mode, change modes, send responses / receive other commands.
3) Checking all devices for their behaviour. Disconnecting the device and sending a notification if malicious device was suspected

The user-space module will provide:
1) Device Logs - shows the exact time of the event, the hardware component involved (e.g., USB port, storage controller), details from the system about the detected device.
2) Safe Devices List – shows user-approved devices that won’t be tracked as unrecognized.
3) Commands Space - an interactive input area where users can enter commands to control and manage devices.

Sketch of how the CLI will look

<img width="818" alt="CLI" src="https://github.com/user-attachments/assets/a09c85a2-ea63-4568-b793-3e45e0337f41" />



## Dependencies
### Kernel module
Firstly to create a kernel module we have to install the 
nightly toolchain.
```bash
rustup install nightly
rustup component add rust-src --toolchain nightly
```
Also we have to install the linux headers since we will use some
libraries from Linux Kernel.
```bash
sudo apt install raspberrypi-kernel-headers
```
Then we have to download bindgen tool for rust. By this tool
it is possible to bind C headers to Rust modules.
```bash![rust_proj.drawio.svg](../../../Downloads/rust_proj.drawio.svg)
cargo install bindgen
```
And in the **cargo.toml** we have to write this dependency:
```
[dependencies]
kernel = { git = "https://github.com/Rust-for-Linux/linux", branch = "rust" }
```
### User-space CLI
#### Desktop Notifications
To enable desktop notifications in Unix-based systems, we need to install the notify-rust library. It allows sending system notifications through the DBus notification daemon, which is commonly used in Linux desktop environments.
```bash
sudo apt install libdbus-1-dev
```
And add it to the Cargo.toml:
```bash
[dependencies]
notify-rust = "4.8"
```
#### Terminal Interaction
To handle terminal output, text formatting, and user input processing, We will use the **crossterm** library. This library provides essential functions for controlling the terminal, such as handling colored text output, cursor movement, and key event detection.
```bash
[dependencies]
crossterm = "0.27"
```

# Diagram of project
<img width="336" alt="Screenshot 2025-03-18 at 14 15 49" src="https://github.com/user-attachments/assets/b1d96af8-3651-42a0-8768-73953ad1f14b" />



```mermaid
graph TD;
    A[Notification mechanism] --> B[CLI interface];
    B --> C[Kernel module];
    C --> D[Polling devices (Lockdown mode)];
    C --> E[Checking new connections (Safe mode)];

