mod cli;
mod compress;
mod config;
mod filters;
mod git_diff;
mod report;
mod transforms;
mod utils;

use cli::{parse_args, print_help, print_preamble};
use compress::compress;
use config::HELP_FLAGS;
use git_diff::process_git_diff;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();

    if arguments.iter().any(|argument| {
        HELP_FLAGS.contains(&argument.as_str())
    }) {
        print_help();
        return;
    }

    if arguments.len() < 2 {
        println!(
            "Нет аргументов. Используйте --help для справки."
        );
        return;
    }

    let config = parse_args(&arguments);

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
