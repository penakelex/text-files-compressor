use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;

use crate::config::{Config, READ_BUFFER_SIZE};

pub fn is_binary_bytes(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(READ_BUFFER_SIZE)].contains(&0)
}

pub fn is_binary_file(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };

    let mut buffer = [0; READ_BUFFER_SIZE];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        return false;
    };

    is_binary_bytes(&buffer[..bytes_read])
}

pub fn is_binary_in_git(
    base_path: &Path,
    path: &str,
) -> bool {
    let Ok(output) = Command::new("git")
        .args(["show", &format!("HEAD:{path}")])
        .current_dir(base_path)
        .output()
    else {
        return false;
    };

    output.status.success()
        && is_binary_bytes(&output.stdout)
}

pub fn extension_of(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_lowercase())
}

pub fn format_size(bytes: u64) -> String {
    const KILOBYTE: u64 = 1024;
    const MEGABYTE: u64 = 1024 * KILOBYTE;

    match bytes {
        size if size >= MEGABYTE => format!(
            "{:.2} MB",
            size as f64 / MEGABYTE as f64
        ),
        size if size >= KILOBYTE => format!(
            "{:.2} KB",
            size as f64 / KILOBYTE as f64
        ),
        size => format!("{size} B"),
    }
}

pub fn output_file_size(config: &Config) -> u64 {
    fs::metadata(&config.output_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}
