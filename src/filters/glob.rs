use globset::{GlobBuilder, GlobSetBuilder};

use super::names::normalize_path;
use crate::config::FilterMatchers;

pub(crate) const GLOB_CHARS: [char; 6] =
    ['*', '?', '[', ']', '{', '}'];

pub(crate) fn is_glob_pattern(pattern: &str) -> bool {
    pattern.contains(GLOB_CHARS)
}

fn glob_from(pattern: &str) -> Result<globset::Glob, String> {
    GlobBuilder::new(pattern).literal_separator(true).build().map_err(
        |error| {
            format!("Невалидный glob-паттерн '{pattern}': {error}")
        },
    )
}

pub fn build_matchers(
    excludes: &[String],
    exclude_paths: &[String],
    include_paths: &[String],
) -> Result<FilterMatchers, String> {
    let mut exclude_set = GlobSetBuilder::new();
    let mut exclude_literals = Vec::new();
    for pattern in excludes {
        let normalized = normalize_path(pattern);
        if normalized.is_empty() {
            continue;
        }
        if is_glob_pattern(&normalized) {
            exclude_set.add(glob_from(&normalized)?);
        } else {
            exclude_literals.push(normalized);
        }
    }

    let mut exclude_path_set = GlobSetBuilder::new();
    let mut exclude_path_literals = Vec::new();
    for pattern in exclude_paths {
        let normalized = normalize_path(pattern);
        if normalized.is_empty() {
            continue;
        }
        if is_glob_pattern(&normalized) {
            exclude_path_set.add(glob_from(&normalized)?);
            exclude_path_set
                .add(glob_from(&format!("{normalized}/**"))?);
            if let Some(prefix) = normalized.strip_suffix("/**")
                && !prefix.is_empty()
                && !is_glob_pattern(prefix)
            {
                exclude_path_literals.push(prefix.to_string());
            }
        } else {
            exclude_path_literals.push(normalized);
        }
    }

    let mut include_set = GlobSetBuilder::new();
    let mut include_literals = Vec::new();
    for pattern in include_paths {
        let normalized = normalize_path(pattern);
        if normalized.is_empty() {
            continue;
        }
        if is_glob_pattern(&normalized) {
            include_set.add(glob_from(&normalized)?);
        } else {
            include_literals.push(normalized);
        }
    }

    Ok(FilterMatchers {
        exclude: exclude_set
            .build()
            .map_err(|error| error.to_string())?,
        exclude_path: exclude_path_set
            .build()
            .map_err(|error| error.to_string())?,
        include: include_set
            .build()
            .map_err(|error| error.to_string())?,
        exclude_literals,
        exclude_path_literals,
        include_literals,
    })
}

pub(crate) fn dir_may_match_glob(
    relative: &str,
    pattern: &str,
) -> bool {
    let literal: String = pattern
        .chars()
        .take_while(|character| !GLOB_CHARS.contains(character))
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
