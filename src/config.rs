use std::path::PathBuf;

use globset::GlobSet;

pub const READ_BUFFER_SIZE: usize = 8192;

pub const RLE_CHAR: char = '\t';
pub const INDENT_WIDTH: usize = 4;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum EolMode {
    Auto,
    Lf,
    Crlf,
}

impl EolMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Lf => "lf",
            Self::Crlf => "crlf",
        }
    }
}

pub struct FilterMatchers {
    pub exclude: GlobSet,
    pub exclude_path: GlobSet,
    pub include: GlobSet,
    pub exclude_literals: Vec<String>,
    pub exclude_path_literals: Vec<String>,
    pub include_literals: Vec<String>,
}

pub struct Config {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub extensions: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub excludes: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_paths: Vec<String>,
    pub matchers: FilterMatchers,
    pub git_mode: bool,
    pub indent_rle: bool,
    pub eol_mode: EolMode,
    pub trim_trailing: bool,
    pub collapse_blank_lines: Option<usize>,
}
