use clap::Args;

const DEFAULT_EXCLUDES_RAW: &str =
    include_str!("../../resources/default_excludes.txt");

pub(crate) fn default_excludes() -> Vec<String> {
    DEFAULT_EXCLUDES_RAW
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_lowercase)
        .collect()
}

#[derive(Args)]
#[command(next_help_heading = "ФИЛЬТРАЦИЯ ПУТЕЙ")]
pub struct Filters {
    #[arg(
        long,
        alias = "exts",
        num_args = 1..,
        value_delimiter = ',',
        help = "Разрешённые расширения. Пусто — включить все."
    )]
    pub extensions: Vec<String>,

    #[arg(
        long,
        alias = "exclude-exts",
        num_args = 1..,
        value_delimiter = ',',
        help = "Исключаемые расширения (побеждает над --extensions)."
    )]
    pub exclude_extensions: Vec<String>,

    #[arg(
        long,
        alias = "excludes",
        num_args = 1..,
        value_delimiter = ',',
        help = "Имена директорий и файлов для исключения (на любой глубине). \
                Добавляются к встроенному списку по умолчанию. \
                Поддерживаются glob-шаблоны (*, ?, **, [...], {...})."
    )]
    pub exclude: Vec<String>,

    #[arg(
        long,
        alias = "exclude-path",
        num_args = 1..,
        value_delimiter = ',',
        help = "Исключить пути (файлы/директории) относительно input_path. \
                Поддерживаются glob-шаблоны: *, ?, **, [...], {...}. \
                Пример: --exclude-paths target/**,src/**/test_*,*.log"
    )]
    pub exclude_paths: Vec<String>,

    #[arg(
        long,
        aliases = ["include-dirs", "only-dirs", "only-paths"],
        num_args = 1..,
        value_delimiter = ',',
        help = "Белый список путей (файлы и/или директории). \".\" — корень. \
                Поддерживаются glob-шаблоны: *, ?, **, [...], {...}. \
                Пример: --include-paths src/**,config/*.json"
    )]
    pub include_paths: Vec<String>,
}
