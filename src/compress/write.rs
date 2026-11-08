use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::Config;
use crate::transforms::transform_content;

pub(crate) fn write_file_content(
    root_path: &Path,
    path: &Path,
    bytes: &[u8],
    writer: &mut BufWriter<File>,
    line_count: &mut u64,
    config: &Config,
) -> std::io::Result<bool> {
    let Ok(content) = std::str::from_utf8(bytes) else {
        return Ok(false);
    };

    let display_path = path
        .strip_prefix(root_path)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");

    writeln!(writer, "### {display_path}")?;

    let transformed = transform_content(
        content,
        config.eol_mode,
        config.indent_rle,
        config.trim_trailing,
        config.collapse_blank_lines,
    );

    writer.write_all(transformed.as_bytes())?;

    let mut newlines =
        transformed.bytes().filter(|&byte| byte == b'\n').count()
            as u64;

    if !transformed.is_empty() && !transformed.ends_with('\n') {
        writer.write_all(b"\n")?;
        newlines += 1;
    }

    *line_count += newlines;

    Ok(true)
}
