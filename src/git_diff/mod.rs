mod content;
mod output;
mod status;

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;
use crate::filters::file_passes_filters;
use crate::output::out;
use crate::report::{print_captured_overview, print_skipped};
use crate::utils::{
    abs, format_size, is_binary_file, make_tmp_path, output_file_size,
};

pub(crate) struct GitEntry {
    pub local: String,
    pub repo: String,
}

#[derive(Clone, Copy)]
enum FileKind {
    Regular,
    Symlink,
    Directory,
    Missing,
}

fn classify(path: &Path) -> FileKind {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            FileKind::Symlink
        }
        Ok(meta) if meta.file_type().is_file() => FileKind::Regular,
        Ok(meta) if meta.file_type().is_dir() => FileKind::Directory,
        Ok(_) => FileKind::Missing,
        Err(_) => FileKind::Missing,
    }
}

fn git_repo_root(input_path: &Path) -> std::io::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(input_path)
        .output()
        .map_err(|error| {
            std::io::Error::other(format!(
                "Не удалось запустить `git rev-parse` \
                 (это git-репозиторий?): {error}"
            ))
        })?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "Не удалось найти корень репозитория: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let root = String::from_utf8(output.stdout).map_err(|_| {
        std::io::Error::other("git rev-parse вернул non-UTF-8 путь")
    })?;

    Ok(PathBuf::from(root.trim()))
}

fn compute_input_rel(
    input_path: &Path,
    repo_root: &Path,
) -> Option<String> {
    let input_c = std::fs::canonicalize(input_path).ok()?;
    let repo_c = std::fs::canonicalize(repo_root).ok()?;
    let rel = input_c.strip_prefix(&repo_c).ok()?;
    let text = rel.to_string_lossy().replace('\\', "/");
    if text.is_empty() { None } else { Some(text) }
}

fn to_git_entry(
    repo_path: String,
    input_rel: &Option<String>,
) -> Option<GitEntry> {
    let local = match input_rel {
        None => repo_path.clone(),
        Some(prefix) => {
            let with_slash = format!("{prefix}/");
            let rest = repo_path.strip_prefix(&with_slash)?;
            rest.to_string()
        }
    };
    Some(GitEntry { local, repo: repo_path })
}

fn filter_non_utf8(
    paths: Vec<String>,
    input_rel: &Option<String>,
) -> Vec<String> {
    match input_rel {
        None => paths,
        Some(prefix) => {
            let with_slash = format!("{prefix}/");
            paths
                .into_iter()
                .filter_map(|path| {
                    path.strip_prefix(&with_slash).map(str::to_string)
                })
                .collect()
        }
    }
}

pub fn process_git_diff(config: &Config) -> std::io::Result<()> {
    let repo_root = git_repo_root(&config.input_path)?;
    let input_rel = compute_input_rel(&config.input_path, &repo_root);

    let status_output = Command::new("git")
        .args(["status", "--porcelain", "-z", "-uall"])
        .current_dir(&config.input_path)
        .output()
        .map_err(|error| {
            std::io::Error::other(format!(
                "Не удалось запустить `git status`: {error}"
            ))
        })?;

    if !status_output.status.success() {
        return Err(std::io::Error::other(format!(
            "`git status` завершился с ошибкой: {}",
            String::from_utf8_lossy(&status_output.stderr).trim()
        )));
    }

    let status = status::parse_status(&status_output.stdout);
    let non_utf8_paths =
        filter_non_utf8(status.non_utf8_paths, &input_rel);

    let tmp_path = make_tmp_path(&config.output_path);
    let output_abs = abs(&config.output_path);
    let tmp_abs = abs(&tmp_path);
    let check_output = |repo_path: &str| -> bool {
        let candidate = abs(&repo_root.join(repo_path));
        candidate == output_abs || candidate == tmp_abs
    };

    let mut skipped_binary = Vec::new();
    let mut skipped_symlinks = Vec::new();
    let mut skipped_non_regular = Vec::new();
    let mut captured_added = Vec::new();
    let mut captured_modified = Vec::new();
    let mut captured_deleted = Vec::new();

    for repo_path in status.added {
        let Some(entry) = to_git_entry(repo_path, &input_rel) else {
            continue;
        };
        if check_output(&entry.repo) {
            continue;
        }
        if !file_passes_filters(&entry.local, config) {
            continue;
        }
        match classify(&repo_root.join(&entry.repo)) {
            FileKind::Symlink => skipped_symlinks.push(entry.local),
            FileKind::Directory => {
                skipped_non_regular.push(entry.local)
            }
            FileKind::Missing => captured_added.push(entry),
            FileKind::Regular => {
                if is_binary_file(&repo_root.join(&entry.repo)) {
                    skipped_binary.push(entry.local);
                } else {
                    captured_added.push(entry);
                }
            }
        }
    }

    for repo_path in status.modified {
        let Some(entry) = to_git_entry(repo_path, &input_rel) else {
            continue;
        };
        if check_output(&entry.repo) {
            continue;
        }
        if !file_passes_filters(&entry.local, config) {
            continue;
        }
        match classify(&repo_root.join(&entry.repo)) {
            FileKind::Symlink => skipped_symlinks.push(entry.local),
            FileKind::Directory => {
                skipped_non_regular.push(entry.local)
            }
            FileKind::Missing => captured_modified.push(entry),
            FileKind::Regular => {
                if is_binary_file(&repo_root.join(&entry.repo)) {
                    skipped_binary.push(entry.local);
                } else {
                    captured_modified.push(entry);
                }
            }
        }
    }

    for repo_path in status.deleted {
        let Some(entry) = to_git_entry(repo_path, &input_rel) else {
            continue;
        };
        if check_output(&entry.repo) {
            continue;
        }
        if !file_passes_filters(&entry.local, config) {
            continue;
        }
        captured_deleted.push(entry);
    }

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

    let write_result = output::write_sections(
        &mut writer,
        config,
        &repo_root,
        &captured_added,
        &captured_modified,
        &captured_deleted,
    )
    .and_then(|stats| writer.flush().map(|()| stats));

    let write_stats = match write_result {
        Ok(stats) => stats,
        Err(error) => {
            let _ = fs::remove_file(&tmp_path);
            return Err(error);
        }
    };
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

    let captured_total = captured_added.len()
        + captured_modified.len()
        + captured_deleted.len();
    let total_changed = captured_total
        + skipped_binary.len()
        + skipped_symlinks.len()
        + skipped_non_regular.len()
        + non_utf8_paths.len();
    let output_bytes = output_file_size(config);

    out!(
        r"📊 Статистика изменений:
   Всего изменённых путей (включая бинарные, симлинки, non-UTF-8): {total_changed}
   ➕ Добавлено: {}
   ✏️  Изменено: {}
   🗑️  Удалено: {}
   🚫 Пропущено бинарных файлов: {}
   🔗 Пропущено симлинков: {}
   📁 Пропущено не-файлов: {}
   🔤 Пропущено non-UTF-8 путей: {}
   📝 Блоков с non-UTF-8 содержимым: {}
   💾 Размер отчёта: {} ({output_bytes} bytes)",
        captured_added.len(),
        captured_modified.len(),
        captured_deleted.len(),
        skipped_binary.len(),
        skipped_symlinks.len(),
        skipped_non_regular.len(),
        non_utf8_paths.len(),
        write_stats.non_utf8,
        format_size(output_bytes),
    );

    print_skipped(
        &skipped_binary,
        "      ",
        "Пропущенные бинарные файлы",
    );
    print_skipped(
        &skipped_symlinks,
        "      ",
        "Пропущенные симлинки",
    );
    print_skipped(
        &skipped_non_regular,
        "      ",
        "Пропущенные не-файлы",
    );
    print_skipped(
        &non_utf8_paths,
        "      ",
        "Пропущенные non-UTF-8 пути",
    );

    out!("✅ Successfully captured {captured_total} files.");

    let all_paths: Vec<&str> = captured_added
        .iter()
        .chain(&captured_modified)
        .chain(&captured_deleted)
        .map(|entry| entry.local.as_str())
        .collect();

    print_captured_overview(&all_paths);

    Ok(())
}
