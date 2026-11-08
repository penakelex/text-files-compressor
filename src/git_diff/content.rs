use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::transforms::transform_content;
use crate::utils::is_binary_bytes;

pub(crate) fn read_file_content(
    base_path: &Path,
    relative_path: &str,
    config: &Config,
) -> Option<String> {
    let mut content = String::new();

    File::open(base_path.join(relative_path))
        .ok()?
        .read_to_string(&mut content)
        .ok()?;

    Some(transform_content(
        &content,
        config.eol_mode,
        config.indent_rle,
        config.trim_trailing,
        config.collapse_blank_lines,
    ))
}

pub(crate) fn read_git_show_content(
    base_path: &Path,
    relative_path: &str,
    config: &Config,
) -> Option<String> {
    let output = Command::new("git")
        .args(["show", &format!("HEAD:{relative_path}")])
        .current_dir(base_path)
        .output()
        .ok()?;

    (output.status.success()
        && !is_binary_bytes(&output.stdout))
    .then(|| String::from_utf8(output.stdout).ok())
    .flatten()
    .map(|content| {
        transform_content(
            &content,
            config.eol_mode,
            config.indent_rle,
            config.trim_trailing,
            config.collapse_blank_lines,
        )
    })
}

pub(crate) fn write_with_newline(
    writer: &mut BufWriter<File>,
    value: &str,
) -> std::io::Result<()> {
    write!(writer, "{value}")?;

    if !value.ends_with('\n') {
        writeln!(writer)?;
    }

    Ok(())
}
