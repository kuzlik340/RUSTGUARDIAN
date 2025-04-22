use whitelist::{create_whitelist_from_connected_devices, is_device_whitelisted};
use notify_rust::Notification;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::HashSet,
    error::Error,
    io,
    process::Command,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub mod whitelist;

enum Event<I> {
    Input(I),
    Tick,
}

enum Focus {
    Logs,
    Whitelist,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(1000);
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let whitelist = create_whitelist_from_connected_devices();
    let mut known_devices = create_whitelist_from_connected_devices();

    let mut logs = vec![
        "[INFO] USB Device Monitor Started",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    for device in &known_devices {
        if !whitelist.contains(device) {
            logs.push(format!("[ALERT] Unrecognized device on startup: '{}'", device));
            Notification::new()
                .summary("USB Device Warning")
                .body(&format!("Unrecognized device detected at startup:\n{}", device))
                .icon("dialog-warning")
                .show()
                .unwrap();
        } else {
            logs.push(format!("[INFO] Whitelisted device on startup: '{}'", device));
        }
    }


    let mut logs = vec![
        "[INFO] USB Device Monitor Started",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let devices = whitelist.iter().cloned().collect::<Vec<_>>();
    let mut input = String::new();
    let mut scroll_offset: usize = 0;
    let mut whitelist_scroll_offset: usize = 0;
    let max_visible_lines: usize = 23;
    let max_visible_whitelist: usize = 25;
    let mut focus = Focus::Logs;

    loop {
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

            let visible_devices = devices
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
                    }));

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
                    logs.push(format!("> {}", input));
                    scroll_offset = logs.len().saturating_sub(max_visible_lines);
                    if input.trim() == ":q" || input.trim() == "exit" {
                        break;
                    }
                    input.clear();
                }
                KeyCode::Up => match focus {
                    Focus::Logs => {
                        if scroll_offset > 0 {
                            scroll_offset -= 1;
                        }
                    }
                    Focus::Whitelist => {
                        if whitelist_scroll_offset > 0 {
                            whitelist_scroll_offset -= 1;
                        }
                    }
                },
                KeyCode::Down => match focus {
                    Focus::Logs => {
                        if scroll_offset + max_visible_lines < logs.len() {
                            scroll_offset += 1;
                        }
                    }
                    Focus::Whitelist => {
                        if whitelist_scroll_offset + max_visible_whitelist < devices.len() {
                            whitelist_scroll_offset += 1;
                        }
                    }
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
                let current_devices = create_whitelist_from_connected_devices();
                for device in &current_devices {
                    if !known_devices.contains(device) {
                        logs.push(format!("[NEW DEVICE] {}", device));
                        if !whitelist.contains(device) {
                            logs.push(format!("[ALERT] Unknown device '{}' connected!", device));
                            Notification::new()
                                .summary("USB Device Alert")
                                .body(&format!("Unknown device connected:\n{}", device))
                                .icon("dialog-warning")
                                .show()
                                .unwrap();
                        } else {
                            logs.push(format!("[INFO] Whitelisted device '{}' connected.", device));
                        }
                    }
                }
                known_devices = current_devices;
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