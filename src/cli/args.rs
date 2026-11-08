use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use super::filters::{Filters, default_excludes};
use super::help::{AFTER_HELP, HELP_TEMPLATE};
use super::paths::default_output_name;
use super::transforms::Transforms;
use crate::config::Config;

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
    #[arg(
        index = 1,
        value_name = "INPUT_PATH",
        help = "Путь к папке проекта. Должен быть первым аргументом после подкоманды."
    )]
    pub input_path: PathBuf,

    #[arg(
        long,
        help = "Имя выходного файла. По умолчанию: <имя input_path>.txt \
                для compress и <имя input_path>.git-diff.txt для git-diff."
    )]
    pub out: Option<String>,

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

        let excludes = if exclude.is_empty() {
            default_excludes()
        } else {
            exclude
        };

        let output_path = self.out.unwrap_or_else(|| {
            default_output_name(&self.input_path, git_mode)
        });

        Config {
            input_path: self.input_path,
            output_path,
            extensions,
            exclude_extensions,
            excludes,
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
