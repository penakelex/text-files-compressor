mod walk;
mod write;

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::config::Config;
use crate::output::out;
use crate::report::{print_captured_overview, print_skipped};
use crate::utils::{
    abs, format_size, make_tmp_path, output_file_size,
};

pub(crate) struct CompressStats {
    pub line_count: u64,
    pub skipped_binary: Vec<String>,
    pub skipped_non_utf8: Vec<String>,
    pub skipped_symlinks: Vec<String>,
    pub captured: Vec<String>,
}

pub fn compress(config: &Config) -> std::io::Result<()> {
    let tmp_path = make_tmp_path(&config.output_path);
    let skip_abs: Vec<PathBuf> =
        vec![abs(&config.output_path), abs(&tmp_path)];

    let file = File::create(&tmp_path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "Не удалось создать {}: {error}",
                tmp_path.display()
            ),
        )
    })?;
    let mut writer = BufWriter::new(file);
    let mut stats = CompressStats {
        line_count: 0,
        skipped_binary: Vec::new(),
        skipped_non_utf8: Vec::new(),
        skipped_symlinks: Vec::new(),
        captured: Vec::new(),
    };

    let walk_result = walk::process_directory(
        &config.input_path,
        &mut writer,
        config,
        &skip_abs,
        &mut stats,
    );

    let finish = walk_result.and_then(|()| writer.flush());

    if let Err(error) = finish {
        let _ = fs::remove_file(&tmp_path);
        return Err(error);
    }
    drop(writer);

    fs::rename(&tmp_path, &config.output_path).map_err(|error| {
        let _ = fs::remove_file(&tmp_path);
        std::io::Error::new(
            error.kind(),
            format!(
                "Не удалось переименовать {} в {}: {error}",
                tmp_path.display(),
                config.output_path.display()
            ),
        )
    })?;

    let output_bytes = output_file_size(config);

    out!(
        r"✅ Successfully captured {} files.
📏 Output content lines (after transforms): {}
🚫 Skipped binary files: {}
🔤 Skipped non-UTF-8 files: {}
🔗 Skipped symlinks: {}
💾 Output size: {} ({output_bytes} bytes)",
        stats.captured.len(),
        stats.line_count,
        stats.skipped_binary.len(),
        stats.skipped_non_utf8.len(),
        stats.skipped_symlinks.len(),
        format_size(output_bytes),
    );

    print_skipped(
        &stats.skipped_binary,
        "   ",
        "Пропущенные бинарные файлы",
    );
    print_skipped(
        &stats.skipped_non_utf8,
        "   ",
        "Пропущенные non-UTF-8 файлы",
    );
    print_skipped(
        &stats.skipped_symlinks,
        "   ",
        "Пропущенные симлинки",
    );

    let references: Vec<&str> =
        stats.captured.iter().map(String::as_str).collect();

    print_captured_overview(&references);

    Ok(())
}
