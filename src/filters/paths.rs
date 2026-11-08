use crate::config::FilterMatchers;

use super::glob::{dir_may_match_glob, is_glob_pattern};
use super::names::normalize_path;

pub fn is_path_excluded(
    relative_path: &str,
    matchers: &FilterMatchers,
) -> bool {
    if matchers.exclude_path_literals.is_empty()
        && matchers.exclude_path.is_empty()
    {
        return false;
    }
    let relative = normalize_path(relative_path);

    let literal_match =
        matchers.exclude_path_literals.iter().any(|pattern| {
            relative == *pattern
                || relative.starts_with(&format!("{pattern}/"))
        });

    if literal_match {
        return true;
    }

    matchers.exclude_path.is_match(&relative)
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
                || include.starts_with(&format!("{relative}/"))
                || relative.starts_with(&format!("{include}/"))
        }
    })
}

pub fn file_in_included_dir(
    relative_file: &str,
    matchers: &FilterMatchers,
) -> bool {
    if matchers.include_literals.is_empty()
        && matchers.include.is_empty()
    {
        return true;
    }

    let relative = normalize_path(relative_file);

    let literal_match =
        matchers.include_literals.iter().any(|include| {
            relative == *include
                || relative.starts_with(&format!("{include}/"))
        });

    literal_match || matchers.include.is_match(&relative)
}
