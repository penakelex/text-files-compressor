use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use crate::config::Config;
use crate::transforms::transform_content;

pub(crate) fn write_file_content(
    root_path: &Path,
    path: &Path,
    writer: &mut BufWriter<File>,
    line_count: &mut u64,
    config: &Config,
) -> std::io::Result<()> {
    let display_path = path
        .strip_prefix(root_path)
        .unwrap_or(path)
        .display();

    writeln!(writer, "### {display_path}")?;

    let mut content = String::new();

    File::open(path)?.read_to_string(&mut content)?;

    let transformed = transform_content(
        &content,
        config.eol_mode,
        config.indent_rle,
        config.trim_trailing,
        config.collapse_blank_lines,
    );

    writer.write_all(transformed.as_bytes())?;

    *line_count += transformed
        .bytes()
        .filter(|&byte| byte == b'\n')
        .count() as u64;

    if !transformed.ends_with('\n') {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
