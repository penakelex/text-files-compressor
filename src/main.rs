mod cli;
mod compress;
mod config;
mod filters;
mod git_diff;
mod output;
mod report;
mod transforms;
mod utils;

use clap::Parser;
use cli::{Cli, Command, print_preamble};
use compress::compress;
use git_diff::process_git_diff;

fn main() {
    let cli = Cli::parse();

    let config = match cli.command {
        Command::Compress(args) => args.into_config(false),
        Command::GitDiff(args) => args.into_config(true),
    };

    let config = match config {
        Ok(config) => config,
        Err(error) => {
            eprintln!("❌ {error}");
            std::process::exit(1);
        }
    };

    if !config.input_path.is_dir() {
        eprintln!(
            "❌ Input path is not a directory: {}",
            config.input_path.display()
        );
        std::process::exit(1);
    }

    print_preamble(&config);

    let result = if config.git_mode {
        process_git_diff(&config)
    } else {
        compress(&config)
    };

    if let Err(error) = result {
        eprintln!("❌ Error: {error}");
        std::process::exit(1);
    }
}
