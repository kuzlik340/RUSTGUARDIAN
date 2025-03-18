# <h1 align="center">RustGuardian</h1>
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
- **Aliaksei Zimnitski** – xzimnitski@stiba.sk
- **Timofei Kuzin** – xkuzin@stuba.sk


## Introduction

The goal of this project is to develop a **kernel module** with a **user-friendly interface** that monitors all **USB devices** and blocks them if malicious activity is detected.
Basically, this project solves a problem with the badUSB attacks that are quite popular nowadays. 

The project consists of two main components:

1. **Kernel Module** – This module operates at the **kernel level**, continuously monitoring USB devices to track their activity. For example, it will check whether a **USB flash drive remains a storage device** or has re-enumerated as an **HID device** (such as a keyboard).

2. **User-Space Module** – This component runs in **user space** and provides a **CLI interface** along with **real-time notifications** when a potentially harmful USB device is detected.

Through this project, we will get experience with Rust’s memory safety features, concurrency, and system-level programming, while building security-focused project to keep badUSB devices away from the OS. 

## Requirements
The kernel module itself will provide:
1) Two or more modes to work. Lockdown mode which will open scan all devices without any sleep. Also a mode which will just compare if connected device is in the safe list.
2) The user-space API: Adding/Deleting safe devices, start Lockdown mode, change modes, send responses / receive other commands.
3) Checking all devices for their behaviour. Disconnecting the device and sending a notification if malicious device was suspected.

The user-space module will provide:
1) Device Logs - shows the exact time of the event, the hardware component involved (e.g., USB port, storage controller), details from the system about the detected device.
2) Safe Devices List – shows user-approved devices that won’t be tracked as unrecognized.
3) Commands Space - an interactive input area where users can enter commands to control and manage devices.

Sketch of how the CLI will look

<img width="818" alt="CLI" src="https://github.com/user-attachments/assets/a09c85a2-ea63-4568-b793-3e45e0337f41" />

The command space will support various commands, some of which are yet to be defined in the project. However, the following are the essential ones for this stage of development:
```
> add device [device_id]  # Will add device to a safe list
> del device [device_id]  # Will delete device from safe list  
> enable LockDown         # Will enable safest mode for polling all devices
> enable SafeConnection   # Will enable mode that will only check if the device that was connected is in safe mode 
> disable LockDown        # Will disable polling
> disable SafeConnection  # Will disable checking all devices is they are in safe list
```



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
it is possible to bind Linux headers to Rust modules.
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
