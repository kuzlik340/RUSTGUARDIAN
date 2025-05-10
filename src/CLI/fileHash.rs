use std::{fs, path::Path};
use walkdir::WalkDir;
use sha2::{Sha256, Digest};

fn hash_file(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Some(format!("{:x}", result))
}

pub fn hash_all_files_in_dir(dir: &Path) -> Vec<(String, String)> {
    let mut hashes = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Ok(data) = fs::read(path) {
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let result = hasher.finalize();
                let hash_str = format!("{:x}", result);
                hashes.push((path.display().to_string(), hash_str));
            }
        }
    }
    hashes
}
