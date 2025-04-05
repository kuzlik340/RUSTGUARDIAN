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
    collections::HashSet,
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

// Whitelist implementation
fn create_whitelist() -> HashSet<String> {
    let mut whitelist = HashSet::new();
    whitelist.insert("QEMU USB HARDDRIVE".to_string());
    whitelist.insert("Generic Keyboard 1234".to_string());
    whitelist
}

fn is_device_whitelisted(device_name: &str, whitelist: &HashSet<String>) -> bool {
    whitelist.contains(device_name)
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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

    let mut logs = vec![
        "[2.433949] usb 1-4.1 Product: QEMU USB HARDDRIVE",
        "[2.433950] usb 1-4.1 Manufacturer: QEMU",
        "[2.433952] usb 1-4.1 SerialNumber: 1-0000:00:02.0-2.1",
        "[2.481181] usb-storage 1-2.1 USB Mass Storage device...",
    ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let mut input = String::new();

    // Initialize whitelist
    let whitelist = create_whitelist();
    let test_device = "QEMU USB HARDDRIVE";
    if is_device_whitelisted(test_device, &whitelist) {
        logs.push(format!("[INFO] Device '{}' is whitelisted", test_device));
    } else {
        logs.push(format!("[WARNING] Device '{}' is NOT whitelisted", test_device));
    }

    // Convert whitelist into a displayable Vec<String>
    let devices = whitelist.iter().cloned().collect::<Vec<_>>();

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

            let log_text = logs
                .iter()
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();
            let log_block = Paragraph::new(log_text)
                .block(Block::default().title("Device logs").borders(Borders::ALL));

            let command_block = Paragraph::new(input.as_ref())
                .block(Block::default().title("Commands").borders(Borders::ALL));

            let device_text = devices
                .iter()
                .map(|l| Spans::from(Span::raw(l)))
                .collect::<Vec<_>>();
            let device_block = Paragraph::new(device_text)
                .block(Block::default().title("Safe devices (Whitelist)").borders(Borders::ALL));

            f.render_widget(log_block, vertical_chunks[0]);
            f.render_widget(command_block, vertical_chunks[1]);
            f.render_widget(device_block, chunks[1]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    logs.push(format!("> {}", input));
                    if input.trim() == ":q" || input.trim() == "exit" {
                        break;
                    }
                    input.clear();
                }
                KeyCode::Esc => break,
                _ => {}
            },
            Event::Tick => {}
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
