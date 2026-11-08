use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::{Config, EolMode, RLE_CHAR};

const HELP_TEMPLATE: &str = "\
{before-help}{about-with-newline}
Использование: {usage}

{all-args}{after-help}
";

const AFTER_HELP: &str = "\
\n\x1b[1m\x1b[4mФОРМАТ ВЫВОДА:\x1b[0m
    Каждый файл начинается с '### path'. Пустых строк между файлами нет.
    --indent-rle: 4 пробела отступа → один символ RLE ('\\t').
    --eol-mode:   auto — как в исходнике, lf — все '\\n', crlf — все '\\r\\n'.
    --collapse-blank-lines N: не более N подряд пустых строк (0 — убрать все).

\x1b[1m\x1b[4mФОРМАТ СПИСКОВ:\x1b[0m
    Списки передаются через запятую или повторением флага:
        --extensions kt,kts,xml
        --extensions kt --extensions kts

\x1b[1m\x1b[4mПРИМЕРЫ:\x1b[0m
    script compress ./project --out full.txt \\
        --include-paths src/main,config/campaign.example.json \\
        --exclude-paths gradle/wrapper,build \\
        --rle --eol-mode lf --trim-trailing --collapse-blank-lines 1

    script git-diff ./project --out diff.txt \\
        --extensions java,kts,xml,json,toml,md,yml,yaml
";

#[derive(Parser)]
#[command(
    name = "text-files-compressor",
    version,
    about = "📘 File Compressor — собирает содержимое исходников в один файл.",
    help_template = HELP_TEMPLATE,
    after_help = AFTER_HELP,
    subcommand_required = true,
    arg_required_else_help = true,
    subcommand_help_heading = "Команды",
    next_help_heading = "Опции",
    disable_help_subcommand = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help, help = "Показать эту справку.")]
    help: Option<bool>,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version, help = "Показать версию.")]
    version: Option<bool>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(
        about = "Собрать содержимое файлов проекта в один файл."
    )]
    Compress(CommonArgs),

    #[command(
        name = "git-diff",
        about = "Сформировать отчёт по git diff (added / modified / deleted)."
    )]
    GitDiff(CommonArgs),
}

#[derive(Args)]
#[command(
    disable_help_flag = true,
    help_template = HELP_TEMPLATE,
    after_help = AFTER_HELP,
    next_help_heading = "ОСНОВНЫЕ ПАРАМЕТРЫ"
)]
pub struct CommonArgs {
    #[arg(help = "Путь к папке проекта.")]
    pub input_path: PathBuf,

    #[arg(
        long,
        default_value = "compress_output.txt",
        help = "Имя выходного файла."
    )]
    pub out: String,

    #[command(flatten)]
    pub filters: Filters,

    #[command(flatten)]
    pub transforms: Transforms,

    #[arg(
        short = 'h',
        long = "help",
        action = clap::ArgAction::Help,
        help = "Показать эту справку.",
        help_heading = "Опции"
    )]
    help: Option<bool>,
}

#[derive(Args)]
#[command(next_help_heading = "ФИЛЬТРАЦИЯ ПУТЕЙ")]
pub struct Filters {
    #[arg(
        long,
        alias = "exts",
        value_delimiter = ',',
        help = "Разрешённые расширения. Пусто — включить все."
    )]
    pub extensions: Vec<String>,

    #[arg(
        long,
        alias = "exclude-exts",
        value_delimiter = ',',
        help = "Исключаемые расширения (побеждает над --extensions)."
    )]
    pub exclude_extensions: Vec<String>,

    #[arg(
        long,
        alias = "excludes",
        value_delimiter = ',',
        default_value = "target,.git,node_modules,.idea,out,build",
        help = "Имена директорий для исключения (на любой глубине)."
    )]
    pub exclude: Vec<String>,

    #[arg(
        long,
        alias = "exclude-path",
        value_delimiter = ',',
        help = "Исключить пути (файлы/директории) относительно input_path."
    )]
    pub exclude_paths: Vec<String>,

    #[arg(
        long,
        aliases = ["include-dirs", "only-dirs", "only-paths"],
        value_delimiter = ',',
        help = "Белый список путей (файлы и/или директории). \".\" — корень."
    )]
    pub include_paths: Vec<String>,
}

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

impl CommonArgs {
    pub fn into_config(self, git_mode: bool) -> Config {
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

        Config {
            input_path: self.input_path,
            output_path: self.out,
            extensions,
            exclude_extensions,
            excludes: exclude,
            exclude_paths,
            include_paths,
            git_mode,
            indent_rle,
            eol_mode,
            trim_trailing,
            collapse_blank_lines,
        }
    }
}

pub fn print_preamble(config: &Config) {
    println!(
        "🚀 Analyzing: {}",
        config.input_path.display()
    );
    println!(
        "🔧 Mode: {}",
        if config.git_mode {
            "Git Diff"
        } else {
            "All Files"
        }
    );
    println!("📦 Output: {}", config.output_path);
    println!("📄 Extensions: {:?}", config.extensions);
    println!(
        "🚫 Excluded extensions: {:?}",
        config.exclude_extensions
    );
    println!("🚫 Exclude names: {:?}", config.excludes);
    println!(
        "🚫 Exclude paths: {:?}",
        config.exclude_paths
    );
    println!(
        "📂 Include paths: {:?}",
        config.include_paths
    );
    println!(
        "🎨 Indent RLE: {}",
        if config.indent_rle {
            format!("on ({RLE_CHAR:?})")
        } else {
            "off".into()
        }
    );
    println!("📝 EOL: {}", config.eol_mode.label());
    println!(
        "✂️  Trim trailing: {}",
        if config.trim_trailing { "on" } else { "off" }
    );
    println!(
        "📉 Blank lines: {}",
        config.collapse_blank_lines.map_or_else(
            || "off".into(),
            |lines| format!("max {lines}"),
        )
    );
    println!("-----------------------------------");
}
