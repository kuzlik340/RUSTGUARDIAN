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

## Requirments
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



