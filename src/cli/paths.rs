use std::path::{Path, PathBuf};

pub(crate) fn resolve_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn display_path(path: &Path) -> String {
    let resolved = resolve_path(path);
    let text = resolved.to_string_lossy();

    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}

pub(crate) fn default_output_name(
    input: &Path,
    git_mode: bool,
) -> String {
    let resolved = resolve_path(input);

    let stem = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("compress_output");

    if git_mode {
        format!("{stem}.git-diff.txt")
    } else {
        format!("{stem}.txt")
    }
}
