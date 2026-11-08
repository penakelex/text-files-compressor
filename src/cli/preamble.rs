use crate::config::{Config, RLE_CHAR};

use super::paths::display_path;

pub fn print_preamble(config: &Config) {
    println!(
        "🚀 Analyzing: {}",
        display_path(&config.input_path)
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
