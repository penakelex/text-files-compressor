use crate::config::Config;
use crate::output::out;

use super::paths::display_path;

fn fmt_list(items: &[String]) -> String {
    if items.is_empty() {
        "(empty)".to_string()
    } else {
        items.join(", ")
    }
}

pub fn print_preamble(config: &Config) {
    out!("🚀 Analyzing: {}", display_path(&config.input_path));
    out!(
        "🔧 Mode: {}",
        if config.git_mode { "Git Diff" } else { "All Files" }
    );
    out!("📦 Output: {}", config.output_path.display());
    out!("📄 Extensions: {}", fmt_list(&config.extensions));
    out!(
        "🚫 Excluded extensions: {}",
        fmt_list(&config.exclude_extensions)
    );
    out!("📁 Exclude names: {}", fmt_list(&config.excludes));
    out!("⛔ Exclude paths: {}", fmt_list(&config.exclude_paths));
    out!("📂 Include paths: {}", fmt_list(&config.include_paths));
    out!(
        "🎨 Indent RLE: {}",
        if config.indent_rle { "on (tab)" } else { "off" }
    );
    out!("📝 EOL: {}", config.eol_mode.label());
    out!(
        "✂️  Trim trailing: {}",
        if config.trim_trailing { "on" } else { "off" }
    );
    out!(
        "📉 Blank lines: {}",
        config.collapse_blank_lines.map_or_else(
            || "off".into(),
            |lines| format!("max {lines}"),
        )
    );
    out!("-----------------------------------");
}
