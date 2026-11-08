use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::{Config, READ_BUFFER_SIZE};

pub fn abs(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn make_tmp_path(output: &Path) -> PathBuf {
    let mut os = output.as_os_str().to_owned();
    os.push(".tmp");
    PathBuf::from(os)
}

pub fn is_binary_bytes(bytes: &[u8]) -> bool {
    bytes.iter().take(READ_BUFFER_SIZE).any(|&byte| byte == 0)
}

pub fn is_binary_file(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            return false;
        }
        Err(error) => {
            eprintln!(
                "⚠️  Не удалось открыть {}: {error}",
                path.display()
            );
            return false;
        }
    };

    let mut buffer = [0; READ_BUFFER_SIZE];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(error) => {
            eprintln!(
                "⚠️  Не удалось прочитать {}: {error}",
                path.display()
            );
            return false;
        }
    };

    is_binary_bytes(&buffer[..bytes_read])
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
        size if size >= MEGABYTE => {
            format!("{:.2} MB", size as f64 / MEGABYTE as f64)
        }
        size if size >= KILOBYTE => {
            format!("{:.2} KB", size as f64 / KILOBYTE as f64)
        }
        size => format!("{size} B"),
    }
}

pub fn output_file_size(config: &Config) -> u64 {
    fs::metadata(&config.output_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}
