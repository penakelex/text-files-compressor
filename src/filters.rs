use crate::config::Config;
use crate::utils::extension_of;

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
    let normalized_path =
        path.to_lowercase().replace('\\', "/");

    excludes.iter().any(|excluded| {
        let excluded = excluded.to_lowercase();

        normalized_path.contains(&format!("/{excluded}/"))
            || normalized_path
                .starts_with(&format!("{excluded}/"))
            || normalized_path == excluded
    })
}

pub fn is_path_excluded(
    relative_path: &str,
    exclude_paths: &[String],
) -> bool {
    if exclude_paths.is_empty() {
        return false;
    }

    let relative =
        relative_path.replace('\\', "/").to_lowercase();

    exclude_paths.iter().any(|pattern| {
        relative == *pattern
            || relative.starts_with(&format!("{pattern}/"))
    })
}

pub fn dir_on_way_or_inside(
    relative_directory: &str,
    includes: &[String],
) -> bool {
    if includes.is_empty() {
        return true;
    }

    let relative = relative_directory
        .replace('\\', "/")
        .to_lowercase();

    includes.iter().any(|include| {
        include.is_empty()
            || include == "."
            || include == "./"
            || relative == *include
            || include.starts_with(&format!("{relative}/"))
            || relative.starts_with(&format!("{include}/"))
    })
}

pub fn file_in_included_dir(
    relative_file: &str,
    includes: &[String],
) -> bool {
    if includes.is_empty() {
        return true;
    }

    let relative =
        relative_file.replace('\\', "/").to_lowercase();

    includes.iter().any(|include| {
        include.is_empty()
            || include == "."
            || include == "./"
            || relative == *include
            || relative.starts_with(&format!("{include}/"))
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
