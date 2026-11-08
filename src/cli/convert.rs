use std::collections::HashSet;
use std::path::PathBuf;

use super::args::CommonArgs;
use super::filters::{Filters, default_excludes};
use super::paths::default_output_name;
use super::transforms::Transforms;
use crate::config::Config;
use crate::filters::{build_matchers, normalize_path};

impl CommonArgs {
    pub fn into_config(
        self,
        git_mode: bool,
    ) -> Result<Config, String> {
        let Filters {
            extensions,
            exclude_extensions,
            exclude,
            exclude_paths,
            include_paths,
        } = self.filters;
        let Transforms {
            indent_rle,
            eol_mode,
            trim_trailing,
            collapse_blank_lines,
        } = self.transforms;

        let extensions = normalize_extensions(extensions);
        let exclude_extensions =
            normalize_extensions(exclude_extensions);

        let mut excludes = default_excludes();
        for item in exclude {
            let lower = normalize_path(&item);
            if !lower.is_empty() && !excludes.contains(&lower) {
                excludes.push(lower);
            }
        }

        let exclude_paths = normalize_patterns(exclude_paths);
        let include_paths = normalize_patterns(include_paths);

        let output_path = self.out.unwrap_or_else(|| {
            PathBuf::from(default_output_name(
                &self.input_path,
                git_mode,
            ))
        });

        let matchers = build_matchers(
            &excludes,
            &exclude_paths,
            &include_paths,
        )?;

        Ok(Config {
            input_path: self.input_path,
            output_path,
            extensions,
            exclude_extensions,
            excludes,
            exclude_paths,
            include_paths,
            matchers,
            git_mode,
            indent_rle,
            eol_mode,
            trim_trailing,
            collapse_blank_lines,
        })
    }
}

fn normalize_extensions(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .map(|item| {
            let trimmed = item.trim();
            trimmed
                .strip_prefix('.')
                .unwrap_or(trimmed)
                .to_lowercase()
        })
        .filter(|item| !item.is_empty())
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

fn normalize_patterns(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .map(|item| normalize_path(&item))
        .filter(|item| !item.is_empty())
        .collect()
}
