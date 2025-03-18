# RUST_PROJECT
## Project Team
- **Aliaksei Zimnitski** – [User-space CLI]
- **Timofei Kuzin** – [Kernel module]


## Introduction

The goal of this project is to develop a **kernel module** with a **user-friendly interface** that monitors all **USB devices** and blocks them if malicious activity is detected.

The project consists of two main components:

1. **Kernel Module** – This module operates at the **kernel level**, continuously monitoring USB devices to track their activity. For example, it will check whether a **USB flash drive remains a storage device** or has re-enumerated as an **HID device** (such as a keyboard).

2. **User-Space Module** – This component runs in **user space** and provides a **CLI interface** along with **real-time notifications** when a potentially harmful USB device is detected.

Through this project, we want to get experience with Rust’s memory safety features, concurrency, and system-level programming, while building something useful and security-focused! 

## Requirements
The kernel module itself will provide:
1) Safety (No kernel panics and glitches while running in all modes)
2) Two or more modes to work. Lockdown mode which will open scan all devices without any sleep. Also a mode which will just compare if connected device is in the safe list.
3) The user-space API: Adding/Deleting safe devices, start Lockdown mode, change modes, send/receive other commands.

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
```bash
cargo install bindgen
```
And in the **cargo.toml** we have to write this dependency:
```
[dependencies]
kernel = { git = "https://github.com/Rust-for-Linux/linux", branch = "rust" }
```



