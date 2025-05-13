mod engine;
mod CLI;
use std::thread;
use std::sync::{Arc, Mutex};
use engine::find_device::find_all_devices;
use CLI::cli::run_cli;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn start_find_device() {
    let find_thread = thread::spawn(|| {
        //println!("Starting thread");
        if let Err(e) = find_all_devices() {
            eprintln!("Find device error: {:?}", e);
        }
    });
}

pub fn push_log(msg: String) {
    let mut logs = LOGS.lock().unwrap();
    logs.push(msg);
}

pub fn get_logs() -> Vec<String> {
    let mut logs = LOGS.lock().unwrap();
    let current_logs = logs.clone();  
    logs.clear();                
    current_logs                     
}

fn main() {
    // Запускаем CLI в отдельном потоке
    let cli_thread = thread::spawn(|| {
        if let Err(e) = run_cli() {
            eprintln!("CLI error: {:?}", e);
        }
    });

    // А в основном потоке запускаем мониторинг
    //find_all_devices();

    // Ждём завершения CLI, если find_all_devices когда-то завершится
    let _ = cli_thread.join();
}