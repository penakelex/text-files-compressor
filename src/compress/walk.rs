use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::filters::{
    dir_on_way_or_inside, file_passes_filters, is_excluded,
    is_path_excluded,
};
use crate::utils::{abs, is_binary_bytes};

use super::CompressStats;
use super::write::write_file_content;

pub(crate) fn process_directory(
    current_path: &Path,
    writer: &mut BufWriter<File>,
    config: &Config,
    skip_abs: &[PathBuf],
    stats: &mut CompressStats,
) -> std::io::Result<()> {
    let entries_iter = match fs::read_dir(current_path) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!(
                "⚠️  Не удалось прочитать {}: {error}",
                current_path.display()
            );
            return Ok(());
        }
    };

    let mut entries = Vec::new();
    for entry in entries_iter {
        match entry {
            Ok(entry) => entries.push(entry),
            Err(error) => {
                eprintln!(
                    "⚠️  Ошибка чтения записи в {}: {error}",
                    current_path.display()
                );
            }
        }
    }
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();

        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        let file_type = meta.file_type();

        let relative_path = path
            .strip_prefix(&config.input_path)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");

        if file_type.is_symlink() {
            stats.skipped_symlinks.push(relative_path);
            continue;
        }

        let candidate_abs = abs(&path);
        if skip_abs.contains(&candidate_abs) {
            continue;
        }

        if file_type.is_dir() {
            if is_excluded(&relative_path, &config.matchers)
                || is_path_excluded(&relative_path, &config.matchers)
                || !dir_on_way_or_inside(
                    &relative_path,
                    &config.include_paths,
                )
            {
                continue;
            }

            process_directory(
                &path, writer, config, skip_abs, stats,
            )?;

            continue;
        }

        if !file_type.is_file()
            || !file_passes_filters(&relative_path, config)
        {
            continue;
        }

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!(
                    "⚠️  Не удалось прочитать {}: {error}",
                    path.display()
                );
                continue;
            }
        };

        if is_binary_bytes(&bytes) {
            stats.skipped_binary.push(relative_path);
            continue;
        }

        match write_file_content(
            &config.input_path,
            &path,
            &bytes,
            writer,
            &mut stats.line_count,
            config,
        ) {
            Ok(true) => stats.captured.push(relative_path),
            Ok(false) => stats.skipped_non_utf8.push(relative_path),
            Err(error) => {
                eprintln!(
                    "⚠️  Ошибка записи {}: {error}",
                    path.display()
                );
            }
        }
    }

    Ok(())
}
