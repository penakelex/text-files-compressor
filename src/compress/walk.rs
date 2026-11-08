use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use crate::config::Config;
use crate::filters::{
    dir_on_way_or_inside, file_passes_filters, is_excluded,
    is_path_excluded,
};
use crate::utils::is_binary_file;

use super::CompressStats;
use super::write::write_file_content;

pub(crate) fn process_directory(
    current_path: &Path,
    writer: &mut BufWriter<File>,
    config: &Config,
    stats: &mut CompressStats,
) -> std::io::Result<()> {
    let Ok(entries) = fs::read_dir(current_path) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let relative_path = path
            .strip_prefix(&config.input_path)
            .unwrap_or(&path)
            .display()
            .to_string();

        if path.is_dir() {
            if is_excluded(&relative_path, &config.excludes)
                || is_path_excluded(
                    &relative_path,
                    &config.exclude_paths,
                )
                || !dir_on_way_or_inside(
                    &relative_path,
                    &config.include_paths,
                )
            {
                continue;
            }

            process_directory(
                &path, writer, config, stats,
            )?;

            continue;
        }

        if !path.is_file()
            || !file_passes_filters(&relative_path, config)
        {
            continue;
        }

        if is_binary_file(&path) {
            stats.skipped_binary.push(relative_path);

            continue;
        }

        let captured = write_file_content(
            &config.input_path,
            &path,
            writer,
            &mut stats.line_count,
            config,
        )?;

        if captured {
            stats.captured.push(relative_path);
        } else {
            stats.skipped_binary.push(relative_path);
        }
    }

    Ok(())
}
