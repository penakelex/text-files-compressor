use std::path::PathBuf;

pub const READ_BUFFER_SIZE: usize = 8192;
pub const DEFAULT_EXCLUDES: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    ".idea",
    "out",
    "build",
];
pub const HELP_FLAGS: &[&str] = &["-h", "--help"];

pub const RLE_CHAR: char = '\t';
pub const INDENT_WIDTH: usize = 4;

#[derive(Clone, Copy)]
pub enum EolMode {
    Auto,
    Lf,
    Crlf,
}

impl EolMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "lf" => Some(Self::Lf),
            "crlf" => Some(Self::Crlf),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Lf => "lf",
            Self::Crlf => "crlf",
        }
    }
}

pub struct Config {
    pub input_path: PathBuf,
    pub output_path: String,
    pub extensions: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub excludes: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_paths: Vec<String>,
    pub git_mode: bool,
    pub indent_rle: bool,
    pub eol_mode: EolMode,
    pub trim_trailing: bool,
    pub collapse_blank_lines: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input_path: PathBuf::new(),
            output_path: "compress_output.txt".to_string(),
            extensions: Vec::new(),
            exclude_extensions: Vec::new(),
            excludes: DEFAULT_EXCLUDES.iter().map(|name| name.to_string()).collect(),
            exclude_paths: Vec::new(),
            include_paths: Vec::new(),
            git_mode: false,
            indent_rle: false,
            eol_mode: EolMode::Auto,
            trim_trailing: false,
            collapse_blank_lines: None,
        }
    }
}