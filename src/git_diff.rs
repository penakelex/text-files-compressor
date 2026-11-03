use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::filters::file_passes_filters;
use crate::report::{
    print_captured_overview, print_skipped_binary,
};
use crate::transforms::transform_content;
use crate::utils::{
    format_size, is_binary_bytes, is_binary_file,
    is_binary_in_git, output_file_size,
};

fn unquote_git_path(path: &str) -> String {
    if path.len() >= 2
        && path.starts_with('"')
        && path.ends_with('"')
    {
        path[1..path.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        path.to_string()
    }
}

fn read_file_content(
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

fn read_git_show_content(
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

pub fn process_git_diff(
    config: &Config,
) -> std::io::Result<()> {
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&config.input_path)
        .output()
        .expect("Failed to run git status. Is this a git repository?");

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    for line in
        String::from_utf8_lossy(&status_output.stdout)
            .lines()
    {
        if line.len() < 4 {
            continue;
        }

        let status = &line[0..2];
        let raw_path = line[3..].trim();

        if status.contains(['R', 'C']) {
            if let Some((old_path, new_path)) =
                raw_path.split_once(" -> ")
            {
                deleted.push(unquote_git_path(old_path));
                added.push(unquote_git_path(new_path));
            }

            continue;
        }

        let path = unquote_git_path(raw_path);

        if status.contains(['?', 'A']) {
            added.push(path);
        } else if status.contains(['M', 'T']) {
            modified.push(path);
        } else if status.contains('D') {
            deleted.push(path);
        }
    }

    let partition_existing = |paths: Vec<String>| -> (
        Vec<String>,
        Vec<String>,
    ) {
        paths
            .into_iter()
            .filter(|path| {
                file_passes_filters(path, config)
            })
            .partition(|path| {
                !is_binary_file(
                    &config.input_path.join(path),
                )
            })
    };

    let (captured_added, mut skipped_binary) =
        partition_existing(added);
    let (captured_modified, skipped_modified) =
        partition_existing(modified);

    skipped_binary.extend(skipped_modified);

    let (captured_deleted, skipped_deleted): (
        Vec<_>,
        Vec<_>,
    ) = deleted
        .into_iter()
        .filter(|path| file_passes_filters(path, config))
        .partition(|path| {
            !is_binary_in_git(&config.input_path, path)
        });

    skipped_binary.extend(skipped_deleted);

    let file = File::create(&config.output_path)?;
    let mut writer = BufWriter::new(file);

    if !captured_added.is_empty() {
        writeln!(writer, "# ADDED")?;

        for path in &captured_added {
            writeln!(writer, "### {path}")?;

            if let Some(content) = read_file_content(
                &config.input_path,
                path,
                config,
            ) {
                write_with_newline(&mut writer, &content)?;
            }
        }

        writeln!(writer)?;
    }

    if !captured_modified.is_empty() {
        writeln!(writer, "# MODIFIED")?;

        for path in &captured_modified {
            writeln!(writer, "### {path}")?;
            writeln!(writer, ">>> current")?;

            if let Some(content) = read_file_content(
                &config.input_path,
                path,
                config,
            ) {
                write_with_newline(&mut writer, &content)?;
            }

            writeln!(writer, "<<< previous")?;

            match read_git_show_content(
                &config.input_path,
                path,
                config,
            ) {
                Some(content) => write_with_newline(
                    &mut writer,
                    &content,
                )?,
                None => writeln!(writer, "<n/a>")?,
            }
        }

        writeln!(writer)?;
    }

    if !captured_deleted.is_empty() {
        writeln!(writer, "# DELETED")?;

        for path in &captured_deleted {
            writeln!(writer, "### {path}")?;
        }

        writeln!(writer)?;
    }

    writer.flush()?;

    let total = captured_added.len()
        + captured_modified.len()
        + captured_deleted.len()
        + skipped_binary.len();
    let output_bytes = output_file_size(config);

    println!(
        r"-----------------------------------
📊 Статистика изменений:
   Всего файлов разнится: {total}
   ➕ Добавлено: {}
   ✏️  Изменено: {}
   🗑️  Удалено: {}
   🚫 Пропущено бинарных файлов: {}
   💾 Размер отчёта: {} ({output_bytes} bytes)",
        captured_added.len(),
        captured_modified.len(),
        captured_deleted.len(),
        skipped_binary.len(),
        format_size(output_bytes),
    );

    print_skipped_binary(&skipped_binary, "      ");

    println!(
        "✅ Successfully captured {} files.",
        captured_added.len()
            + captured_modified.len()
            + captured_deleted.len()
    );

    let all_paths: Vec<&str> = captured_added
        .iter()
        .chain(&captured_modified)
        .chain(&captured_deleted)
        .map(String::as_str)
        .collect();

    print_captured_overview(&all_paths);

    Ok(())
}
