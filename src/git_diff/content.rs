use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::transforms::transform_content;
use crate::utils::is_binary_bytes;

pub(crate) enum FileContent {
    Content(String),
    Binary,
    NonUtf8,
    Missing,
}

pub(crate) fn read_file_content(
    base_path: &Path,
    relative_path: &str,
    config: &Config,
) -> FileContent {
    let path = base_path.join(relative_path);
    let Ok(bytes) = std::fs::read(&path) else {
        return FileContent::Missing;
    };

    if is_binary_bytes(&bytes) {
        return FileContent::Binary;
    }

    match String::from_utf8(bytes) {
        Ok(content) => FileContent::Content(transform_content(
            &content,
            config.eol_mode,
            config.indent_rle,
            config.trim_trailing,
            config.collapse_blank_lines,
        )),
        Err(_) => FileContent::NonUtf8,
    }
}

pub(crate) fn read_git_show_content(
    base_path: &Path,
    relative_path: &str,
    config: &Config,
) -> FileContent {
    let output = Command::new("git")
        .args(["show", &format!("HEAD:{relative_path}")])
        .current_dir(base_path)
        .output();

    let Ok(output) = output else {
        return FileContent::Missing;
    };

    if !output.status.success() {
        return FileContent::Missing;
    }

    if is_binary_bytes(&output.stdout) {
        return FileContent::Binary;
    }

    let Ok(content) = String::from_utf8(output.stdout) else {
        return FileContent::NonUtf8;
    };

    FileContent::Content(transform_content(
        &content,
        config.eol_mode,
        config.indent_rle,
        config.trim_trailing,
        config.collapse_blank_lines,
    ))
}
