use std::{fs, path::Path};
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::push_log;

// This function will load hashes of viruses so the program could
// compare them with the hashes on flash drive
pub fn load_hashes_from_file(path: &str) -> HashSet<String> {
    let file = File::open(path).expect("Cannot open hash file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_lowercase()) 
        .collect()
}

/// Computes the SHA-256 hash of a single file at the given path.
/// Returns the hexadecimal string representation of the hash.
fn hash_file(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?; // Read file contents
    let mut hasher = Sha256::new(); // Create SHA-256 hasher
    hasher.update(&data); // Feed file data into the hasher
    let result = hasher.finalize(); // Finalize and get the hash
    Some(format!("{:x}", result)) // Return hash as hex string
}



/// Recursively walks through all files in the given directory,
/// computes SHA-256 hash for each file, and returns a vector of (file_path, hash) pairs.
pub fn hash_all_files_in_dir(dir: &Path, hash_set: &HashSet<String>) -> Vec<(String, String)> {
    let mut hashes = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(hash) = hash_file(path) {
                let hash_clone = hash.clone(); 
                hashes.push((path.display().to_string(), hash_clone));
                if hash_set.contains(&hash) {
                    push_log(format!("File with path: {} is malicious", path.display().to_string() ))
                }
            }
        }
    }

    hashes
}
