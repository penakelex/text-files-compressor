mod walk;
mod write;

use std::fs::File;
use std::io::BufWriter;

use crate::config::Config;
use crate::report::{
    print_captured_overview, print_skipped_binary,
};
use crate::utils::{format_size, output_file_size};

pub(crate) struct CompressStats {
    pub line_count: u64,
    pub skipped_binary: Vec<String>,
    pub captured: Vec<String>,
}

pub fn compress(config: &Config) -> std::io::Result<()> {
    let file = File::create(&config.output_path)?;
    let mut writer = BufWriter::new(file);
    let mut stats = CompressStats {
        line_count: 0,
        skipped_binary: Vec::new(),
        captured: Vec::new(),
    };

    walk::process_directory(
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
