use whitelist::{create_whitelist_from_connected_devices, is_device_whitelisted};


// Crossterm for terminal control and input events
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Standard library imports
use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

// TUI (Text-based UI) components
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub mod whitelist;

// Custom event enum: either keyboard input or timer tick
enum Event<I> {
    Input(I),
    Tick,
}

// UI focus state: either on logs or whitelist
enum Focus {
    Logs,
    Whitelist,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal in raw mode and enable alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Channel for sending events (keyboard/timer) to main loop
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();
        loop {
            // Poll for input events with timeout
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            // Send Tick event on interval
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    // Initialize whitelist and test device
    let whitelist = create_whitelist_from_connected_devices();
    let mut logs = vec![
        "[2.433949] usb 1-4.1 Product: QEMU USB HARDDRIVE",
        "[2.433950] usb 1-4.1 Manufacturer: QEMU",
        "[2.433952] usb 1-4.1 SerialNumber: 1-0000:00:02.0-2.1",
        "[2.481181] usb-storage 1-2.1 USB Mass Storage device...",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // Log whether test device is whitelisted
    let test_device = "QEMU USB HARDDRIVE";
    if is_device_whitelisted(test_device, &whitelist) {
        logs.push(format!("[INFO] Device '{}' is whitelisted", test_device));
    } else {
        logs.push(format!("[WARNING] Device '{}' is NOT whitelisted", test_device));
    }

    let devices = whitelist.iter().cloned().collect::<Vec<_>>();
    let mut input = String::new();

    // Scroll positions and limits
    let mut scroll_offset: usize = 0;
    let mut whitelist_scroll_offset: usize = 0;
    let max_visible_lines: usize = 23;
    let max_visible_whitelist: usize = 25;

    // Currently focused UI panel
    let mut focus = Focus::Logs;

    // Main event/render loop
    loop {
        terminal.draw(|f| {
            // Divide terminal area into layout chunks
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

            // Prepare visible logs (with scroll)
            let visible_logs = logs
                .iter()
                .skip(scroll_offset)
                .take(max_visible_lines)
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();

            // Prepare visible whitelist devices (with scroll)
            let visible_devices = devices
                .iter()
                .skip(whitelist_scroll_offset)
                .take(max_visible_whitelist)
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();

            // Log area block (highlighted if focused)
            let log_block = Paragraph::new(visible_logs)
                .block(Block::default()
                    .title("Device logs")
                    .borders(Borders::ALL)
                    .style(if matches!(focus, Focus::Logs) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }));

            // Input area
            let command_block = Paragraph::new(input.as_ref())
                .block(Block::default().title("Commands").borders(Borders::ALL));

            // Whitelist block (highlighted if focused)
            let device_block = Paragraph::new(visible_devices)
                .block(Block::default()
                    .title("Safe devices (Whitelist)")
                    .borders(Borders::ALL)
                    .style(if matches!(focus, Focus::Whitelist) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }));

            // Render all blocks
            f.render_widget(log_block, vertical_chunks[0]);
            f.render_widget(command_block, vertical_chunks[1]);
            f.render_widget(device_block, chunks[1]);
        })?;

        // Handle keyboard and tick events
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => input.push(c),          // Add character to input buffer
                KeyCode::Backspace => { input.pop(); },     // Remove last character
                KeyCode::Enter => {
                    // Log command and auto-scroll down
                    logs.push(format!("> {}", input));
                    scroll_offset = logs.len().saturating_sub(max_visible_lines);

                    // Exit command
                    if input.trim() == ":q" || input.trim() == "exit" {
                        break;
                    }

                    // Clear input
                    input.clear();
                }
                KeyCode::Up => match focus {
                    // Scroll up in current focused block
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
                    // Scroll down in current focused block
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
                    // Switch focus between logs and whitelist
                    focus = match focus {
                        Focus::Logs => Focus::Whitelist,
                        Focus::Whitelist => Focus::Logs,
                    };
                }
                KeyCode::Esc => break, // Exit on Escape
                _ => {}
            },
            Event::Tick => {} // Tick event can be used for future animations or updates
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
