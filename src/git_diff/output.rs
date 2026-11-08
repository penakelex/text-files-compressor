use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::Config;

use super::GitEntry;
use super::content::{
    FileContent, read_file_content, read_git_show_content,
};

#[derive(Default)]
pub(crate) struct WriteStats {
    pub non_utf8: usize,
}

pub(crate) fn write_sections(
    writer: &mut BufWriter<File>,
    config: &Config,
    repo_root: &Path,
    added: &[GitEntry],
    modified: &[GitEntry],
    deleted: &[GitEntry],
) -> std::io::Result<WriteStats> {
    let mut stats = WriteStats::default();
    write_added(writer, config, repo_root, added, &mut stats)?;
    write_modified(writer, config, repo_root, modified, &mut stats)?;
    write_deleted(writer, deleted)?;
    Ok(stats)
}

fn write_with_newline(
    writer: &mut BufWriter<File>,
    value: &str,
) -> std::io::Result<()> {
    write!(writer, "{value}")?;
    if !value.ends_with('\n') {
        writeln!(writer)?;
    }
    Ok(())
}

fn write_content(
    writer: &mut BufWriter<File>,
    content: FileContent,
    stats: &mut WriteStats,
) -> std::io::Result<()> {
    match content {
        FileContent::Content(value) => {
            write_with_newline(writer, &value)
        }
        FileContent::Binary => writeln!(writer, "<binary>"),
        FileContent::NonUtf8 => {
            stats.non_utf8 += 1;
            writeln!(writer, "<non-UTF-8>")
        }
        FileContent::Missing => writeln!(writer, "<n/a>"),
    }
}

fn write_added(
    writer: &mut BufWriter<File>,
    config: &Config,
    repo_root: &Path,
    added: &[GitEntry],
    stats: &mut WriteStats,
) -> std::io::Result<()> {
    if added.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# ADDED")?;

    for entry in added {
        writeln!(writer, "### {}", entry.local)?;
        let content =
            read_file_content(repo_root, &entry.repo, config);
        write_content(writer, content, stats)?;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_modified(
    writer: &mut BufWriter<File>,
    config: &Config,
    repo_root: &Path,
    modified: &[GitEntry],
    stats: &mut WriteStats,
) -> std::io::Result<()> {
    if modified.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# MODIFIED")?;

    for entry in modified {
        let current =
            read_file_content(repo_root, &entry.repo, config);
        let previous =
            read_git_show_content(repo_root, &entry.repo, config);

        writeln!(writer, "### {}", entry.local)?;
        writeln!(writer, ">>> current")?;
        write_content(writer, current, stats)?;

        writeln!(writer, "<<< previous")?;
        write_content(writer, previous, stats)?;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_deleted(
    writer: &mut BufWriter<File>,
    deleted: &[GitEntry],
) -> std::io::Result<()> {
    if deleted.is_empty() {
        return Ok(());
    }

    writeln!(writer, "# DELETED")?;

    for entry in deleted {
        writeln!(writer, "### {}", entry.local)?;
    }

    writeln!(writer)?;
    Ok(())
}
