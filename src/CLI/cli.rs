use super::whitelist::create_whitelist_from_connected_devices;
use super::filehash::{hash_all_files_in_dir, load_hashes_from_file};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use notify_rust::Notification;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
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
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crate::get_logs;
use crate::DeviceMonitor;
use crate::start_hash_checker;
use crate::push_log;
use crate::WHITELIST;


enum Event<I> {
    Input(I),
    Tick,
}

enum Focus {
    Logs,
    Whitelist,
}

pub fn folders(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|r| r.path())
        .filter(|r| r.is_dir())
        .collect())
}

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut pending_mounts: HashSet<String> = HashSet::new();
    let mut scanned_paths: HashSet<PathBuf> = HashSet::new();

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

    // Use global whitelist
    let mut known_devices: HashSet<String> = {
        let whitelist = WHITELIST.read().unwrap();
        whitelist.keys().cloned().collect()
    };
    

    let mut logs = vec!["[INFO] USB Device Monitor Started".to_string()];

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

    let mut input = String::new();
    let mut scroll_offset: usize = 0;
    let mut whitelist_scroll_offset: usize = 0;
    let max_visible_lines: usize = 23;
    let max_visible_whitelist: usize = 25;
    let mut focus = Focus::Logs;
    let mut device_monitor = DeviceMonitor::new();

    loop {
        logs.extend(get_logs());
        let whitelist_vec = {
            let wl = WHITELIST.read().unwrap();
            wl.iter()
                .map(|(id, name)| format!("[{}] {}", id, name))
                .collect::<Vec<String>>()
        };
        
        
        

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

            let log_block = Paragraph::new(visible_logs)
                .block(Block::default()
                    .title("Device logs")
                    .borders(Borders::ALL)
                    .style(if matches!(focus, Focus::Logs) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }))
                .wrap(Wrap { trim: true });

            let command_block = Paragraph::new(input.as_ref())
                .block(Block::default()
                    .title("Commands")
                    .borders(Borders::ALL)
                    .style(if matches!(focus, Focus::Logs) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }));

            let device_block = Paragraph::new(visible_devices)
                .block(Block::default()
                    .title("Safe devices (Whitelist)")
                    .borders(Borders::ALL)
                    .style(if matches!(focus, Focus::Whitelist) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }));

            f.render_widget(log_block, vertical_chunks[0]);
            f.render_widget(command_block, vertical_chunks[1]);
            f.render_widget(device_block, chunks[1]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => { input.pop(); },
                KeyCode::Enter => {
                    push_log(format!("> {}", input));
                    scroll_offset = logs.len().saturating_sub(max_visible_lines);
                    if input.trim() == ":q" || input.trim() == "exit" {
                        break;
                    }
                    input.clear();
                }
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
                // Сканируем подключенные устройства: HashMap<ID, Name>
                let current_devices = create_whitelist_from_connected_devices();
            
                for (id, name) in &current_devices {
                    let whitelist = WHITELIST.read().unwrap();
            
                    if !whitelist.contains_key(id) && !known_devices.contains(id) {
                        push_log(format!("[ALERT] Unknown device [{}] {} connected!", id, name));
                        Notification::new()
                            .summary("USB Device Alert")
                            .body(&format!("Unknown device connected:\n[{}] {}", id, name))
                            .icon("dialog-warning")
                            .show()
                            .ok();
                    }
                }
            
                // Обновляем known_devices
                known_devices = current_devices.keys().cloned().collect();
            
                // Проверка на смонтированные пути
                let user_mount_path = format!(
                    "/media/{}",
                    std::env::var("SUDO_USER")
                        .or_else(|_| std::env::var("USER"))
                        .unwrap_or_else(|_| "debian".to_string())
                );
                if let Ok(entries) = folders(Path::new(&user_mount_path)) {
                    for path in entries {
                        if !scanned_paths.contains(&path) {
                            push_log(format!("[MOUNT DETECTED] {:?}", path));
                            start_hash_checker(&path);
                            scanned_paths.insert(path);
                        }
                    }
                } else {
                    push_log(format!("[INFO] No devices in {} yet", user_mount_path));
                }
            }
                     
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
