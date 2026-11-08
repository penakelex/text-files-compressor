use crate::config::FilterMatchers;
use crate::utils::extension_of;

pub fn normalize_path(path: &str) -> String {
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

pub fn matches_extensions(path: &str, extensions: &[String]) -> bool {
    extensions.is_empty()
        || extension_of(path)
            .is_some_and(|ext| extensions.contains(&ext))
}

pub fn extension_is_excluded(
    path: &str,
    exclude_extensions: &[String],
) -> bool {
    extension_of(path)
        .is_some_and(|ext| exclude_extensions.contains(&ext))
}

pub fn is_excluded(path: &str, matchers: &FilterMatchers) -> bool {
    let normalized_path = normalize_path(path);
    if normalized_path.is_empty() {
        return false;
    }

    let segments: Vec<&str> = normalized_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    let literal_match =
        matchers.exclude_literals.iter().any(|excluded| {
            segments.contains(&excluded.as_str())
                || normalized_path == *excluded
                || normalized_path
                    .starts_with(&format!("{excluded}/"))
        });

    if literal_match {
        return true;
    }

    matchers.exclude.is_match(&normalized_path)
        || segments
            .iter()
            .any(|segment| matchers.exclude.is_match(*segment))
}
