use globset::GlobBuilder;

use crate::config::Config;
use crate::utils::extension_of;

const GLOB_CHARS: [char; 6] =
    ['*', '?', '[', ']', '{', '}'];

fn is_glob_pattern(pattern: &str) -> bool {
    pattern.contains(GLOB_CHARS)
}

fn glob_match(pattern: &str, path: &str) -> bool {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|glob| glob.compile_matcher().is_match(path))
        .unwrap_or(false)
}

fn normalize_path(path: &str) -> String {
    let normalized = path
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_matches('/')
        .to_lowercase();

    if normalized.is_empty() || normalized == "." {
        String::new()
    } else {
        normalized
    }
}

pub fn matches_extensions(
    path: &str,
    extensions: &[String],
) -> bool {
    extensions.is_empty()
        || extension_of(path).is_some_and(|extension| {
            extensions.contains(&extension)
        })
}

pub fn extension_is_excluded(
    path: &str,
    exclude_extensions: &[String],
) -> bool {
    extension_of(path).is_some_and(|extension| {
        exclude_extensions.contains(&extension)
    })
}

pub fn is_excluded(
    path: &str,
    excludes: &[String],
) -> bool {
    let normalized_path = normalize_path(path);

    if normalized_path.is_empty() {
        return false;
    }

    let segments: Vec<&str> = normalized_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    excludes.iter().any(|excluded| {
        let excluded = normalize_path(excluded);

        if excluded.is_empty() {
            return false;
        }

        if is_glob_pattern(&excluded) {
            glob_match(&excluded, &normalized_path)
                || segments.iter().any(|segment| {
                    glob_match(&excluded, segment)
                })
        } else {
            segments.contains(&excluded.as_str())
        }
    })
}

pub fn is_path_excluded(
    relative_path: &str,
    exclude_paths: &[String],
) -> bool {
    if exclude_paths.is_empty() {
        return false;
    }

    let relative = normalize_path(relative_path);

    exclude_paths.iter().any(|pattern| {
        let pattern = normalize_path(pattern);

        if pattern.is_empty() {
            return false;
        }

        if is_glob_pattern(&pattern) {
            glob_match(&pattern, &relative)
                || glob_match(
                    &format!("{pattern}/**"),
                    &relative,
                )
        } else {
            relative == pattern
                || relative
                    .starts_with(&format!("{pattern}/"))
        }
    })
}

fn dir_may_match_glob(
    relative: &str,
    pattern: &str,
) -> bool {
    let literal: String = pattern
        .chars()
        .take_while(|character| {
            !GLOB_CHARS.contains(character)
        })
        .collect();

    let dir_prefix = match literal.rfind('/') {
        Some(position) => &literal[..position],
        None => "",
    };

    if dir_prefix.is_empty() {
        return true;
    }

    relative == dir_prefix
        || relative.starts_with(&format!("{dir_prefix}/"))
        || dir_prefix.starts_with(&format!("{relative}/"))
}

pub fn dir_on_way_or_inside(
    relative_directory: &str,
    includes: &[String],
) -> bool {
    if includes.is_empty() {
        return true;
    }

    let relative = normalize_path(relative_directory);

    if relative.is_empty() {
        return true;
    }

    includes.iter().any(|include| {
        let include = normalize_path(include);

        if include.is_empty() {
            return true;
        }

        if is_glob_pattern(&include) {
            dir_may_match_glob(&relative, &include)
        } else {
            relative == include
                || include
                    .starts_with(&format!("{relative}/"))
                || relative
                    .starts_with(&format!("{include}/"))
        }
    })
}

pub fn file_in_included_dir(
    relative_file: &str,
    includes: &[String],
) -> bool {
    if includes.is_empty() {
        return true;
    }

    let relative = normalize_path(relative_file);

    includes.iter().any(|include| {
        let include = normalize_path(include);

        if include.is_empty() {
            return true;
        }

        if is_glob_pattern(&include) {
            glob_match(&include, &relative)
        } else {
            relative == include
                || relative
                    .starts_with(&format!("{include}/"))
        }
    })
}

pub fn file_passes_filters(
    path: &str,
    config: &Config,
) -> bool {
    matches_extensions(path, &config.extensions)
        && !extension_is_excluded(
            path,
            &config.exclude_extensions,
        )
        && file_in_included_dir(path, &config.include_paths)
        && !is_excluded(path, &config.excludes)
        && !is_path_excluded(path, &config.exclude_paths)
}
