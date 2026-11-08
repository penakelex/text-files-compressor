use clap::Args;

use crate::config::EolMode;

#[derive(Args)]
#[command(next_help_heading = "ФОРМАТ ВЫВОДА")]
pub struct Transforms {
    #[arg(
        long = "indent-rle",
        alias = "rle",
        help = "4 пробела отступа → один символ RLE (по умолчанию '\\t')."
    )]
    pub indent_rle: bool,

    #[arg(
        long,
        value_enum,
        ignore_case = true,
        default_value = "auto",
        value_name = "auto|lf|crlf",
        help = "Нормализация переводов строк."
    )]
    pub eol_mode: EolMode,

    #[arg(
        long,
        help = "Срезать пробелы и табы в конце строк."
    )]
    pub trim_trailing: bool,

    #[arg(
        long = "collapse-blank-lines",
        alias = "collapse-blank",
        help = "Не более N пустых строк подряд (0 — удалить все)."
    )]
    pub collapse_blank_lines: Option<usize>,
}
