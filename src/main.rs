mod cli;
mod compress;
mod config;
mod filters;
mod git_diff;
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
