use std::fs::File;
use std::io::{BufWriter, Write};

use crate::config::Config;

use super::content::{
    read_file_content, read_git_show_content,
    write_with_newline,
};

pub(crate) fn write_sections(
    writer: &mut BufWriter<File>,
    config: &Config,
    added: &[String],
    modified: &[String],
    deleted: &[String],
) -> std::io::Result<()> {
    write_added(writer, config, added)?;
    write_modified(writer, config, modified)?;
    write_deleted(writer, deleted)?;

    Ok(())
}

fn write_added(
    writer: &mut BufWriter<File>,
    config: &Config,
    added: &[String],
) -> std::io::Result<()> {
    if added.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# ADDED")?;

    for path in added {
        writeln!(writer, "### {path}")?;

        if let Some(content) = read_file_content(
            &config.input_path,
            path,
            config,
        ) {
            write_with_newline(writer, &content)?;
        }
    }

    writeln!(writer)?;

    Ok(())
}

fn write_modified(
    writer: &mut BufWriter<File>,
    config: &Config,
    modified: &[String],
) -> std::io::Result<()> {
    if modified.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# MODIFIED")?;

    for path in modified {
        writeln!(writer, "### {path}")?;
        writeln!(writer, ">>> current")?;

        if let Some(content) = read_file_content(
            &config.input_path,
            path,
            config,
        ) {
            write_with_newline(writer, &content)?;
        }

        writeln!(writer, "<<< previous")?;

        match read_git_show_content(
            &config.input_path,
            path,
            config,
        ) {
            Some(content) => {
                write_with_newline(writer, &content)?
            }
            None => writeln!(writer, "<n/a>")?,
        }
    }

    writeln!(writer)?;

    Ok(())
}

fn write_deleted(
    writer: &mut BufWriter<File>,
    deleted: &[String],
) -> std::io::Result<()> {
    if deleted.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# DELETED")?;

    for path in deleted {
        writeln!(writer, "### {path}")?;
    }

    writeln!(writer)?;

    Ok(())
}
