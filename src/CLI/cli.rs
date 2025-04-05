use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
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

// Event type: either keyboard input or timer tick
enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Enable raw mode (disable input buffering and echo)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Switch to alternate screen (fullscreen mode)
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create a channel to receive input or tick events
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(250);
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

    // Predefined log messages (can be updated later)
    let mut logs = vec![
        "[2.433949] usb 1-4.1 Product: QEMU USB HARDDRIVE",
        "[2.433950] usb 1-4.1 Manufacturer: QEMU",
        "[2.433952] usb 1-4.1 SerialNumber: 1-0000:00:02.0-2.1",
        "[2.481181] usb-storage 1-2.1 USB Mass Storage device...",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // Fake list of devices
    let devices = vec![
        "sr0     11:0     1  1024M  rom",
        "vda     254:0    0  64G    disk",
        "├─vda1  254:1    0  512M   part /boot/efi",
        "├─vda2  254:2    0  62.5G  part /",
        "└─vda3  254:3    0  976M   part [SWAP]",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let mut input = String::new(); // User input buffer for command entry

    loop {
        // Main TUI drawing block
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

            // Device logs panel
            let log_text = logs
                .iter()
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();
            let log_block = Paragraph::new(log_text)
                .block(Block::default().title("Device logs").borders(Borders::ALL));

            // Command input panel
            let command_block = Paragraph::new(input.as_ref())
                .block(Block::default().title("Commands").borders(Borders::ALL));

            // Safe devices panel
            let device_text = devices
                .iter()
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();
            let device_block = Paragraph::new(device_text)
                .block(Block::default().title("Safe devices").borders(Borders::ALL));

            f.render_widget(log_block, vertical_chunks[0]);
            f.render_widget(command_block, vertical_chunks[1]);
            f.render_widget(device_block, chunks[1]);
        })?;

        // Handle user input or tick events
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => {
                    input.push(c); // Add character to command input
                }
                KeyCode::Backspace => {
                    input.pop(); // Remove last character
                }
                KeyCode::Enter => {
                    logs.push(format!("> {}", input)); // Append command to log panel
                    if input.trim() == ":q" || input.trim() == "exit" {
                        break; // Exit program
                    }
                    input.clear();
                }
                KeyCode::Esc => break, // Exit on Esc
                _ => {}
            },
            Event::Tick => {} // Can be used to update dynamic content
        }
    }

    // Cleanup: restore terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
