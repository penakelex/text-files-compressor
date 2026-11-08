mod glob;
mod names;
mod paths;

pub use glob::build_matchers;
pub use names::{
    extension_is_excluded, is_excluded, matches_extensions,
    normalize_path,
};
pub use paths::{
    dir_on_way_or_inside, file_in_included_dir, is_path_excluded,
};

use crate::config::Config;

pub fn file_passes_filters(path: &str, config: &Config) -> bool {
    matches_extensions(path, &config.extensions)
        && !extension_is_excluded(path, &config.exclude_extensions)
        && file_in_included_dir(path, &config.matchers)
        && !is_excluded(path, &config.matchers)
        && !is_path_excluded(path, &config.matchers)
}
