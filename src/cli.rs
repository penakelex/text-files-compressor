use std::path::{Path, PathBuf};

use crate::config::{Config, EolMode, HELP_FLAGS, INDENT_WIDTH, RLE_CHAR};

fn parse_extensions(raw: &str) -> impl Iterator<Item = String> + '_ {
    raw.split(',')
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
}

fn parse_paths(raw: &str) -> impl Iterator<Item = String> + '_ {
    raw.split(',')
        .map(|item| item.trim().replace('\\', "/").to_lowercase())
        .map(|item| {
            let item = item.trim_matches('/');
            let item = item.strip_prefix("./").unwrap_or(item);

            item.trim_end_matches('/').to_string()
        })
        .filter(|item| !item.is_empty())
}

#[derive(Clone, Copy)]
enum OptionKind {
    Flag,
    SingleValue,
    ListValue,
}

struct OptionSpec {
    name: &'static str,
    aliases: &'static [&'static str],
    kind: OptionKind,
    argument_placeholder: Option<&'static str>,
    help: &'static str,
    preamble_label: Option<&'static str>,
    default_display: Option<&'static str>,
    apply: fn(&mut Config, &str),
    render: fn(&Config) -> String,
}

static OPTIONS: &[OptionSpec] = &[
    OptionSpec {
        name: "--git",
        aliases: &["--diff"],
        kind: OptionKind::Flag,
        argument_placeholder: None,
        help: "Режим работы с git diff.",
        preamble_label: Some("🔧 Mode"),
        default_display: None,
        apply: |config, _| config.git_mode = true,
        render: |config| {
            if config.git_mode {
                "Git Diff".into()
            } else {
                "All Files".into()
            }
        },
    },
    OptionSpec {
        name: "--out",
        aliases: &[],
        kind: OptionKind::SingleValue,
        argument_placeholder: Some("<file>"),
        help: "Имя выходного файла.",
        preamble_label: Some("📦 Output"),
        default_display: Some("compress_output.txt"),
        apply: |config, value| config.output_path = value.to_string(),
        render: |config| config.output_path.clone(),
    },
    OptionSpec {
        name: "--extensions",
        aliases: &["--exts"],
        kind: OptionKind::ListValue,
        argument_placeholder: Some("<list>"),
        help: "Разрешённые расширения. Пусто — включить все.",
        preamble_label: Some("📄 Extensions"),
        default_display: None,
        apply: |config, value| config.extensions.extend(parse_extensions(value)),
        render: |config| format!("{:?}", config.extensions),
    },
    OptionSpec {
        name: "--exclude-extensions",
        aliases: &["--exclude-exts"],
        kind: OptionKind::ListValue,
        argument_placeholder: Some("<list>"),
        help: "Исключаемые расширения (побеждает над --extensions).",
        preamble_label: Some("🚫 Excluded extensions"),
        default_display: None,
        apply: |config, value| config.exclude_extensions.extend(parse_extensions(value)),
        render: |config| format!("{:?}", config.exclude_extensions),
    },
    OptionSpec {
        name: "--exclude",
        aliases: &["--excludes"],
        kind: OptionKind::ListValue,
        argument_placeholder: Some("<names>"),
        help: "Имена директорий для исключения (на любой глубине).",
        preamble_label: Some("🚫 Exclude names"),
        default_display: Some("target, .git, node_modules, .idea, out, build"),
        apply: |config, value| config.excludes.extend(parse_extensions(value)),
        render: |config| format!("{:?}", config.excludes),
    },
    OptionSpec {
        name: "--exclude-paths",
        aliases: &["--exclude-path"],
        kind: OptionKind::ListValue,
        argument_placeholder: Some("<paths>"),
        help: "Исключить пути (файлы/директории) относительно input_path.",
        preamble_label: Some("🚫 Exclude paths"),
        default_display: None,
        apply: |config, value| config.exclude_paths.extend(parse_paths(value)),
        render: |config| format!("{:?}", config.exclude_paths),
    },
    OptionSpec {
        name: "--include-paths",
        aliases: &["--include-dirs", "--only-dirs", "--only-paths"],
        kind: OptionKind::ListValue,
        argument_placeholder: Some("<paths>"),
        help: "Белый список путей (файлы и/или директории). \".\" — корень.",
        preamble_label: Some("📂 Include paths"),
        default_display: None,
        apply: |config, value| config.include_paths.extend(parse_paths(value)),
        render: |config| format!("{:?}", config.include_paths),
    },
    OptionSpec {
        name: "--indent-rle",
        aliases: &["--rle"],
        kind: OptionKind::Flag,
        argument_placeholder: None,
        help: "4 пробела отступа → один символ RLE (по умолчанию '\\t').",
        preamble_label: Some("🎨 Indent RLE"),
        default_display: None,
        apply: |config, _| config.indent_rle = true,
        render: |config| {
            if config.indent_rle {
                format!("on ({RLE_CHAR:?})")
            } else {
                "off".into()
            }
        },
    },
    OptionSpec {
        name: "--eol-mode",
        aliases: &[],
        kind: OptionKind::SingleValue,
        argument_placeholder: Some("<auto|lf|crlf>"),
        help: "Нормализация переводов строк.",
        preamble_label: Some("📝 EOL"),
        default_display: Some("auto"),
        apply: |config, value| {
            config.eol_mode = EolMode::parse(value).unwrap_or_else(|| {
                panic!("invalid --eol-mode: {value} (expected auto|lf|crlf)")
            });
        },
        render: |config| config.eol_mode.label().into(),
    },
    OptionSpec {
        name: "--trim-trailing",
        aliases: &[],
        kind: OptionKind::Flag,
        argument_placeholder: None,
        help: "Срезать пробелы и табы в конце строк.",
        preamble_label: Some("✂️  Trim trailing"),
        default_display: None,
        apply: |config, _| config.trim_trailing = true,
        render: |config| {
            if config.trim_trailing {
                "on".into()
            } else {
                "off".into()
            }
        },
    },
    OptionSpec {
        name: "--collapse-blank-lines",
        aliases: &["--collapse-blank"],
        kind: OptionKind::SingleValue,
        argument_placeholder: Some("<n>"),
        help: "Не более N пустых строк подряд (0 — удалить все).",
        preamble_label: Some("📉 Blank lines"),
        default_display: Some("off"),
        apply: |config, value| {
            config.collapse_blank_lines = Some(value.parse().unwrap_or_else(|_| {
                panic!("--collapse-blank-lines: expected non-negative integer, got {value}")
            }));
        },
        render: |config| {
            config
                .collapse_blank_lines
                .map_or_else(|| "off".into(), |lines| format!("max {lines}"))
        },
    },
];

fn find_spec(argument: &str) -> Option<&'static OptionSpec> {
    OPTIONS
        .iter()
        .find(|spec| spec.name == argument || spec.aliases.contains(&argument))
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/') || value.contains('\\') || Path::new(value).exists()
}

fn consume_list(
    arguments: &[String],
    start_index: usize,
    input_path: &Option<PathBuf>,
) -> (String, usize) {
    let mut end_index = start_index;

    while end_index < arguments.len() && !arguments[end_index].starts_with("--") {
        if input_path.is_none() && looks_like_path(&arguments[end_index]) {
            break;
        }

        end_index += 1;
    }

    let value = arguments[start_index..end_index].join(", ");
    let last_index = if end_index > start_index {
        end_index - 1
    } else {
        start_index.saturating_sub(1)
    };

    (value, last_index)
}

pub fn parse_args(arguments: &[String]) -> Config {
    let mut config = Config::default();
    let mut input_path: Option<PathBuf> = None;
    let mut index = 1;

    while index < arguments.len() {
        let argument = arguments[index].as_str();

        if let Some(spec) = find_spec(argument) {
            match spec.kind {
                OptionKind::Flag => (spec.apply)(&mut config, ""),
                OptionKind::SingleValue => {
                    index += 1;
                    assert!(
                        index < arguments.len(),
                        "{name} requires a value",
                        name = spec.name
                    );
                    (spec.apply)(&mut config, &arguments[index]);
                }
                OptionKind::ListValue => {
                    let (value, new_index) =
                        consume_list(arguments, index + 1, &input_path);
                    (spec.apply)(&mut config, &value);
                    index = new_index;
                }
            }
        } else if !argument.starts_with("--") {
            input_path = Some(PathBuf::from(argument));
        }

        index += 1;
    }

    config.input_path = input_path.expect("Input path is required");

    config
}

pub fn print_preamble(config: &Config) {
    println!("🚀 Analyzing: {}", config.input_path.display());

    for spec in OPTIONS {
        if let Some(label) = spec.preamble_label {
            let value = (spec.render)(config);

            println!("{label}: {value}");
        }
    }

    println!("-----------------------------------");
}

pub fn print_help() {
    let rows: Vec<(String, String)> = OPTIONS
        .iter()
        .map(|spec| {
            let mut header = spec.name.to_string();

            for alias in spec.aliases {
                header.push_str(", ");
                header.push_str(alias);
            }

            if let Some(placeholder) = spec.argument_placeholder {
                header.push(' ');
                header.push_str(placeholder);
            }

            let mut help = spec.help.to_string();

            if let Some(default) = spec.default_display {
                help.push_str(&format!(" (по умолчанию: {default})"));
            }

            (header, help)
        })
        .collect();

    let help_flags = HELP_FLAGS.join(", ");
    let width = rows
        .iter()
        .map(|(header, _)| header.chars().count())
        .chain(std::iter::once(help_flags.chars().count()))
        .max()
        .unwrap_or(0);

    let input_argument = "<input_path>";

    println!(
        r"📘 File Compressor — собирает содержимое исходников в один файл.

USAGE:
    script <input_path> [OPTIONS]

ARGUMENTS:
    {input_argument:<width$} Путь к папке проекта.

OPTIONS:"
    );

    for (header, help) in &rows {
        println!("    {header:<width$} {help}");
    }

    println!("    {help_flags:<width$} Показать эту справку.");

    println!(
        r"
ФИЛЬТРАЦИЯ ПУТЕЙ:
    --exclude <names>       имена директорий (любая глубина).
    --exclude-paths <paths> относительные пути (файл или директория).
                              Директория → весь подкаталог; файл → только он.
    --include-paths <paths> белый список. Пути к файлам и/или директориям.
                              Пример: src/main config/x.json
"
    );

    println!(
        r"
ФОРМАТ СПИСКОВ:
    Списки передаются без кавычек, через запятую или пробел:
        --extensions kt, kts, xml, toml, conf
        --extensions kt kts xml toml conf
"
    );

    println!(
        r"
ФОРМАТ ВЫВОДА:
    Каждый файл начинается с '### path'. Пустых строк между файлами нет.
    --indent-rle: {INDENT_WIDTH} пробела отступа → один символ RLE ({RLE_CHAR:?}).
    --eol-mode:  auto — как в исходнике, lf — все '\n', crlf — все '\r\n'.
    --collapse-blank-lines N: не более N подряд пустых строк (0 — убрать все).
"
    );

    println!(
        r#"
EXAMPLES:
    script "C:\Users\HONOR\Programming\Java\homework-2" \
        --git --out diff.txt \
        --extensions java kts xml json toml md yml yaml \
        --exclude .gradle .gitlab

    script ./project --out full.txt \
        --include-paths src/main config/campaign.example.json \
        --exclude-paths gradle/wrapper build \
        --rle --eol-mode lf --trim-trailing --collapse-blank 1
"#
    );
}