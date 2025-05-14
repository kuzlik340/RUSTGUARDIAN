
use sysinfo::{System, SystemExt, ProcessExt};
use std::collections::HashSet;
use lazy_static::lazy_static;
use sysinfo::PidExt;
use crate::push_log;
use std::thread;
use std::time::Duration;

// List of suspicous names for processes
lazy_static! {
    static ref MALICIOUS_NAMES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("miner");
        set.insert("crypt");
        set.insert("malware");
        set.insert("spyware");
        set.insert("rootkit");
        set.insert("keylogger");
        set.insert("ransom");
        set
    };
}
#[derive(Clone)]
pub struct ProcessScanResult {
    pub pid: i32,
    pub name: String,
    pub is_suspicious: bool,
    pub reason: Option<String>,
}

pub fn scan_processes() -> Vec<ProcessScanResult> {
    push_log(format!("Scanning processes"));
    let mut system = System::new();
    system.refresh_processes();
    
    let mut results = Vec::new();

    for (pid, process) in system.processes() {
        let name = process.name().to_lowercase();
        let mut reason = None;
        let mut is_suspicious = false;

        // Checker by name of process
        if MALICIOUS_NAMES.iter().any(|&mal_name| name.contains(mal_name)) {
            is_suspicious = true;
            reason = Some(format!("Known malicious process name: {}", name));
        }
        // Checker for the process usage
        else if process.cpu_usage() > 90.0 {
            is_suspicious = true;
            reason = Some(format!("High CPU usage: {}%", process.cpu_usage()));
        }

        results.push(ProcessScanResult {
            pid: pid.as_u32() as i32,
            name: process.name().to_string(),
            is_suspicious,
            reason,
        });
    }
    thread::sleep(Duration::from_millis(100));

    results
}