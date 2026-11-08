mod content;
mod output;
mod paths;
mod status;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::Command;

use crate::config::Config;
use crate::filters::file_passes_filters;
use crate::report::{
    print_captured_overview, print_skipped_binary,
};
use crate::utils::{
    format_size, is_binary_file, is_binary_in_git,
    output_file_size,
};

pub fn process_git_diff(
    config: &Config,
) -> std::io::Result<()> {
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&config.input_path)
        .output()
        .expect("Failed to run git status. Is this a git repository?");

    let status =
        status::parse_status(&status_output.stdout);

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
        partition_existing(status.added);
    let (captured_modified, skipped_modified) =
        partition_existing(status.modified);
    skipped_binary.extend(skipped_modified);

    let (captured_deleted, skipped_deleted): (
        Vec<_>,
        Vec<_>,
    ) = status
        .deleted
        .into_iter()
        .filter(|path| file_passes_filters(path, config))
        .partition(|path| {
            !is_binary_in_git(&config.input_path, path)
        });
    skipped_binary.extend(skipped_deleted);

    let file = File::create(&config.output_path)?;
    let mut writer = BufWriter::new(file);

    output::write_sections(
        &mut writer,
        config,
        &captured_added,
        &captured_modified,
        &captured_deleted,
    )?;

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
