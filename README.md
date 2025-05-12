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
- **Aliaksei Zimnitski** – xzimnitski@stuba.sk
- **Timofei Kuzin** – xkuzin@stuba.sk


## Introduction

The goal of this project is to develop a lightweight antivirus with a **CLI interface** that will check USB devices for malicious activity and also processes that are currently running in OS.
Basically, this project solves a problem with the badUSB attacks and USB-based malware infections that are quite popular nowadays.

The project consists of two main components:

1. **Application Core** – In SafeConnection mode, this module monitors all USB device connections and disconnections, comparing each device against a whitelist. If a device is not on the whitelist, it is automatically blocked (users can unblock it via the CLI). In LockDown mode, scanns all files on mountable volumes by sending file hashes to the VirusTotal API, also it will scan some binary documents of processes that are currently running if they will look strange.

2. **CLI interface** – This component provides a **CLI interface** along with **real-time notifications** when a potentially malicious activity is detected.

Through this project, we will get experience with Rust’s memory safety features, concurrency, and system-level programming, while building security-focused project to keep malicious devices away from the OS.

## Requirements
### The **Application Core** itself will provide:
1) Multiple operating modes:
- **LockDown mode**, which will check the operational system for suspicious processes and binaries and also will check mountable devices.
- **SafeConnection mode**, which checks whether the connected device is present in the whitelist and blocks it if not.
- **Background daemons** – Threads responsible for monitoring processes and USB connections.

2) Behavior analysis and response:
- Monitors the behavior of all connected devices.
- Automatically disconnects devices and sends a notification if malicious activity is suspected or if a malicious file is detected on a mountable volume.
- Automatically kills the process if the process is malicious and also sends a notification with the path to the binary of the process.

### The CLI module will provide:
1) Logs - shows the exact time of the event, the hardware component involved (e.g., USB port, storage controller), details from the system about the detected device. Also shows the exact time if a suspicious process was found. 
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
### Application core
To detect when a USB device is connected or disconnected, we use the udev library. This library allows monitoring and retrieving information about devices from user space.
```bash
sudo apt-get install libudev-dev
```
For the sandboxing and intercepting devices will be used evdev:
```bash
sudo apt-get install libevdev-dev
```
Also to connect to the VirusTotal API will be used libssl:
```bash
sudo apt-get install libssl-dev
```


### CLI module
#### Desktop Notifications
To enable desktop notifications in Unix-based systems, we need to install the notify-rust library. It allows sending system notifications through the DBus notification daemon, which is commonly used in Linux desktop environments.
```bash
sudo apt update
sudo apt install libnotify-bin
```
And add it to the Cargo.toml:
```bash
[dependencies]
notify-rust = "4.6"
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
                              │           Application core             │
                              │                                        │
                              │  -Creating sandbox(Lockdown mode)      │
                              │  -Check input from unknown HID-device  │
                              │    (Lockdown mode)                     │
                              │  -Checking new connections(Safe mode)  │
                              │                                        │
                              └────────────────────────────────────────┘
```

# TODO - section for team
- [x] KeyLogger - will be logging all keyboard events. (Assigned to Kuzin)
- [x] Speed checker - BadUSB will input all comands very fast, we have to check the speed of symbols written into terminal. (Assigned to Kuzin)
- [x] CLI - create a CLI that will check the commands in command space and will print out logs (Assigned to Zinmitski)
- [x] Create white-list functions. (Asigned to Zimnitski)
- [x] Run through files on mountable volume and create hashes of files. (Assigned to Zimnitski)
- [x] Notification mechanism - create a system that will notify user if something goes wrong. (Assigned to Zinmitski)
- [ ] Create local storage for hashes of viruses and make a logic for updating itself (Assigned to Kuzin)
- [ ] Connect every module into one (Assigned to Kuzin)
- [ ] Create a module which will go through processes to find malicious software. (Assigned to Kuzin)
- [ ] Create daemons for the corresponding modules of antivirus. (Assigned to Zinmitski)
