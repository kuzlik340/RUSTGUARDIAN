// Imports necessary modules and functions
use super::whitelist::create_whitelist_from_connected_devices;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::fs;
use notify_rust::Notification;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use super::device_functions::{DeviceList, DeviceEntry};
use tui::widgets::Wrap;
use std::{
    collections::HashSet,
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
    path::Path,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Style, Color},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::collections::VecDeque;
use crate::get_logs;
use crate::DeviceMonitor;
use crate::start_hash_checker;
use crate::push_log;
use crate::WHITELIST;
use crate::WHITELIST_READY;


// Enum to differentiate between keyboard input and timed tick events
enum Event<I> {
    Input(I),
    Tick,
}

// Enum to manage which panel is currently focused
enum Focus {
    Logs,
    Whitelist,
}

// Returns a list of directories in the given path
pub fn folders(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|r| r.path())
        .filter(|r| r.is_dir())
        .collect())
}

pub fn all_folders_recursive(root: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut result = vec![];
    let mut queue = VecDeque::new();

    if root.is_dir() {
        queue.push_back(root.to_path_buf());
    }

    while let Some(current_dir) = queue.pop_front() {
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                result.push(path.clone());
                queue.push_back(path);
            }
        }
    }

    Ok(result)
}


fn is_usb_flash(device_id: &str) -> bool {
    let sysfs_path = format!("/sys/bus/usb/devices/{}/bDeviceClass", 
                            device_id.replace(':', "."));
    
    // Читаем класс устройства (0x08 - Mass Storage)
    std::fs::read_to_string(sysfs_path)
        .ok()
        .and_then(|s| u8::from_str_radix(s.trim(), 16).ok())
        .map(|class| class == 0x08)
        .unwrap_or(false)
}

// Entry point for the CLI interface
pub fn run_cli() -> Result<(), Box<dyn Error>> {
    if std::env::var("USER").unwrap_or_default() != "root" {
        push_log("The lockdown mode is disabled, since youn are not root".to_string());
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // DeviceList to hold detected but not whitelisted USB devices
    let mut device_list = DeviceList::new(100);
    let mut safe_connection = false;
    let mut lockdown = false;
    // Channel for communicating between UI input and tick thread
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(1000);
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    let _ = tx.send(Event::Input(key));
                }
            }
            if last_tick.elapsed() >= tick_rate {
                let _ = tx.send(Event::Tick);
                last_tick = Instant::now();
            }
        }
    });

    // Load initial known devices from whitelist
    let mut known_devices: HashSet<String> = {
        let whitelist = WHITELIST.read().unwrap();
        whitelist.keys().cloned().collect()
    };

    let mut logs = vec!["[INFO] USB Device Monitor Started".to_string()];

    // Log any unexpected devices on startup
    for device in &known_devices {
        let whitelist = WHITELIST.read().unwrap();
        if !whitelist.contains_key(device) {
            push_log(format!("[ALERT] Unrecognized device on startup: '{}'", device));
            Notification::new()
                .summary("USB Device Warning")
                .body(&format!("Unrecognized device detected at startup:\n{}", device))
                .icon("dialog-warning")
                .show()
                .ok();

        } else {
            push_log(format!("[INFO] Whitelisted device on startup: '{}'", device));
        }
    }

    // UI and command loop
    let mut input = String::new();
    let mut scroll_offset: usize = 0;
    let mut whitelist_scroll_offset: usize = 0;
    let max_visible_lines: usize = 23;
    let max_visible_whitelist: usize = 25;
    let mut focus = Focus::Logs;
    let mut device_monitor = DeviceMonitor::new();

    loop {
        // Append new logs on every iteration
        logs.extend(get_logs());

        // Render whitelist for display
        let whitelist_vec = {
            let wl = WHITELIST.read().unwrap();
            wl.iter()
                .map(|(id, name)| format!("[{}] {}", id, name))
                .collect::<Vec<String>>()
        };

        // Draw the UI layout
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(chunks[0]);

            let visible_logs = logs
                .iter()
                .skip(scroll_offset)
                .take(max_visible_lines)
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();

            let visible_devices = whitelist_vec
                .iter()
                .skip(whitelist_scroll_offset)
                .take(max_visible_whitelist)
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();

            // Render logs, command input, and whitelist
            let log_block = Paragraph::new(visible_logs)
                .block(
                    Block::default()
                        .title("Device logs")
                        .borders(Borders::ALL)
                        .style(
                            if matches!(focus, Focus::Logs) {
                                Style::default().fg(Color::White)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        )
                )
                .wrap(Wrap { trim: true });

            let command_block = Paragraph::new(input.as_ref())
                .block(
                    Block::default()
                        .title("Commands")
                        .borders(Borders::ALL)
                        .style(
                            if matches!(focus, Focus::Logs) {
                                Style::default().fg(Color::White)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        )
                );

            let device_block = Paragraph::new(visible_devices)
                .block(
                    Block::default()
                        .title("Safe devices (Whitelist)")
                        .borders(Borders::ALL)
                        .style(
                            if matches!(focus, Focus::Whitelist) {
                                Style::default().fg(Color::White)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        )
                );


            f.render_widget(log_block, vertical_chunks[0]);
            f.render_widget(command_block, vertical_chunks[1]);
            f.render_widget(device_block, chunks[1]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => { input.pop(); },
                KeyCode::Enter => {
                    // Log entered command
                    push_log(format!("> {}", input));
                    scroll_offset = logs.len().saturating_sub(max_visible_lines);
                
                    // Command parsing
                    match input.trim() {
                        ":q" | "exit" => {
                            if lockdown{
                                device_monitor.stop();
                            }
                            break;
                        }

                        "dlist" => {
                            device_list.log_devices(); // List all tracked untrusted devices
                        }

                        cmd if cmd.starts_with("wadd ") => {
                            // Add selected device to whitelist
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() == 2 {
                                if let Ok(index) = parts[1].parse::<usize>() {
                                    let maybe_entry = device_list.get(index).cloned();
                                    match maybe_entry {
                                        Some(entry) => {
                                            let mut whitelist = WHITELIST.write().unwrap();
                                            if whitelist.contains_key(&entry.id) {
                                                push_log(format!("> [{}] already in whitelist", entry.id));
                                            } else {
                                                whitelist.insert(entry.id.clone(), entry.name.clone());
                                                push_log(format!("> Added [{}] {} to whitelist", entry.id, entry.name));
                                                if let Err(e) = device_list.remove_device(index) {
                                                    push_log(format!("> Failed to remove device [{}]: {}", entry.id, e));
                                                }
                                            }
                                        }
                                        None => {
                                            push_log(format!("> No device at index {}", index));
                                        }
                                    }
                                } else {
                                    push_log(format!("> Invalid index: {}", parts[1]));
                                }
                            } else {
                                push_log("> Usage: :wadd <index>".to_string());
                            }
                        }

                        "enable LockDown" => {
                            if std::env::var("USER").unwrap_or_default() != "root" {
                                push_log("[SECURITY] You are not root".to_string());
                            }
                            else if !lockdown {
                                lockdown = true;
                                push_log("[SECURITY] LockDown mode enabled".to_string());
                                device_monitor.start();
                            }
                            else{
                                push_log("[SECURITY] LockDown mode was already enabled".to_string());
                            }
                        }
                        
                        "enable SafeConnection" => {
                            push_log("[SECURITY] SafeConnection mode enabled".to_string());
                            safe_connection = true;
                        }

                        "disable LockDown" => {
                            if lockdown {
                                lockdown = false;
                                device_monitor.stop();
                                push_log("[SECURITY] LockDown mode disabled".to_string());
                            }
                            else{
                                push_log("[SECURITY] LockDown was not enabled".to_string());
                            }
                             
                        }

                        "disable SafeConnection" => {
                            push_log("[SECURITY] SafeConnection mode disabled".to_string());
                            safe_connection = false;
                        }

                        _ => {
                            push_log(format!("Unknown command: {}", input.trim()));
                        }
                    }

                    input.clear();
                }

                // Navigation between logs and whitelist
                KeyCode::Up => match focus {
                    Focus::Logs => if scroll_offset > 0 { scroll_offset -= 1; },
                    Focus::Whitelist => if whitelist_scroll_offset > 0 { whitelist_scroll_offset -= 1; },
                },
                KeyCode::Down => match focus {
                    Focus::Logs => if scroll_offset + max_visible_lines < logs.len() { scroll_offset += 1; },
                    Focus::Whitelist => if whitelist_scroll_offset + max_visible_whitelist < whitelist_vec.len() {
                        whitelist_scroll_offset += 1;
                    },
                },
                KeyCode::Tab => {
                    focus = match focus {
                        Focus::Logs => Focus::Whitelist,
                        Focus::Whitelist => Focus::Logs,
                    };
                }
                KeyCode::Esc => break,
                _ => {}
            },

            Event::Tick => {
                // Wait until whitelist has been initialized
                if !WHITELIST_READY.load(Ordering::SeqCst) {
                    continue;
                }

                // Poll USB devices via lsusb
                let current_devices = create_whitelist_from_connected_devices();

                for (id, name) in &current_devices {
                    let whitelist = WHITELIST.read().unwrap();
                    let id_clone = id.clone();
                    let name_clone = name.clone();
                    // If device is unknown, warn and add to tracked list
                    if !whitelist.contains_key(id) && !known_devices.contains(id) {
                        push_log(format!("[ALERT] Unknown device [{}] {} connected!", id_clone, name_clone));
                        let is_flash = is_usb_flash(&id_clone);
                        push_log(format!("IS flash = {}", is_flash));
                        Notification::new()
                            .summary("USB Device Alert")
                            .body(&format!("Unknown device connected:\n[{}] {}", id_clone, name_clone))
                            .icon("dialog-warning")
                            .show()
                            .ok(); 
                        known_devices = current_devices.keys().cloned().collect();

                        if safe_connection {
                            thread::spawn(move || {
                                let user_mount_path = format!(
                                            "/media/{}",
                                            std::env::var("SUDO_USER")
                                                .or_else(|_| std::env::var("USER"))
                                                .unwrap_or_else(|_| "debian".to_string())
                                );

                                push_log(format!("[USB MOUNT DETECTED] {:?}", user_mount_path));
                                // Check for mounted paths in /media
                                let mut scanned_paths: HashSet<PathBuf> = HashSet::new();
                                thread::sleep(Duration::from_secs(2));
                                if let Ok(entries) =  folders(Path::new(&user_mount_path)) {
                                    if entries.is_empty() {
                                        push_log(format!("[INFO] Directory {} is empty", user_mount_path));
                                    }
                                    for path in entries {
                                        if !scanned_paths.contains(&path) {
                                            start_hash_checker(&path);
                                            scanned_paths.insert(path);
                                        }
                                    }
                                } else {
                                    push_log(format!("[INFO] No devices in {} yet", user_mount_path));
                                }
                            });
                        }

                        let entry = DeviceEntry {
                            id: id.clone(),
                            name: name.clone(),
                        };
                        match device_list.add_device(entry) {
                            Ok(index) => {
                                push_log(format!("> Added device [{}] to slot {}", id, index));
                            }
                            Err(e) => {
                                push_log(format!("> Failed to add device [{}]: {}", id, e));
                            }
                        }
                    }
                }

                // Remove disconnected devices from device_list
                let current_ids: HashSet<_> = current_devices.keys().cloned().collect();
                let removed_ids: Vec<String> = known_devices
                    .difference(&current_ids)
                    .cloned()
                    .collect();

                for id in removed_ids {
                    // Try to find and remove device from device_list
                    if let Some(index) = (0..100).find(|&i| device_list.get(i).map(|d| &d.id) == Some(&id)) {
                        if let Err(e) = device_list.remove_device(index) {
                            push_log(format!("> Failed to remove disconnected device [{}]: {}", id, e));
                        } else {
                            push_log(format!("> Device [{}] disconnected and removed from review device list", id));
                        }
                    }
                }


                // Update seen device list
            
        
            }
        }
    }

    // Restore terminal on exit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
