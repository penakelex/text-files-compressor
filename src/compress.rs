use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use crate::config::Config;
use crate::filters::{
    dir_on_way_or_inside, file_passes_filters,
    is_path_excluded,
};
use crate::report::{
    print_captured_overview, print_skipped_binary,
};
use crate::transforms::transform_content;
use crate::utils::{
    format_size, is_binary_file, output_file_size,
};

struct CompressStats {
    line_count: u64,
    skipped_binary: Vec<String>,
    captured: Vec<String>,
}

pub fn compress(config: &Config) -> std::io::Result<()> {
    let file = File::create(&config.output_path)?;
    let mut writer = BufWriter::new(file);
    let mut stats = CompressStats {
        line_count: 0,
        skipped_binary: Vec::new(),
        captured: Vec::new(),
    };

    process_directory(
        &config.input_path,
        &mut writer,
        config,
        &mut stats,
    )?;

    let output_bytes = output_file_size(config);

    println!(
        r"-----------------------------------
✅ Successfully captured {} files.
📏 Total lines: {}
🚫 Skipped binary files: {}
💾 Output size: {} ({output_bytes} bytes)",
        stats.captured.len(),
        stats.line_count,
        stats.skipped_binary.len(),
        format_size(output_bytes),
    );

    print_skipped_binary(&stats.skipped_binary, "   ");

    let references: Vec<&str> =
        stats.captured.iter().map(String::as_str).collect();

    print_captured_overview(&references);

    Ok(())
}

fn process_directory(
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

        let Some(file_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_lowercase)
        else {
            continue;
        };

        let relative_path = path
            .strip_prefix(&config.input_path)
            .unwrap_or(&path)
            .display()
            .to_string();

        if path.is_dir() {
            if config.excludes.contains(&file_name)
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

        stats.captured.push(relative_path);

        write_file_content(
            &config.input_path,
            &path,
            writer,
            &mut stats.line_count,
            config,
        )?;
    }

    Ok(())
}

fn write_file_content(
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
