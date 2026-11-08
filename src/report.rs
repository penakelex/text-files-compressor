use std::collections::BTreeMap;

use crate::output::out;
use crate::utils::extension_of;

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

fn insert_path(root: &mut TreeNode, path: &str) {
    let mut current = root;

    for part in path
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
    {
        current =
            current.children.entry(part.to_string()).or_default();
    }
}

fn print_tree_recursive(node: &TreeNode, prefix: &str) {
    let children_count = node.children.len();

    for (index, (name, child)) in node.children.iter().enumerate() {
        let is_last = index + 1 == children_count;
        let connector =
            if is_last { "└── " } else { "├── " };

        out!("{prefix}{connector}{name}");

        let tail = if is_last { "    " } else { "│   " };
        let new_prefix = format!("{prefix}{tail}");

        print_tree_recursive(child, &new_prefix);
    }
}

fn print_captured_tree(paths: &[&str]) {
    if paths.is_empty() {
        out!("   (нет захваченных файлов)");
        return;
    }

    let mut root = TreeNode::default();
    for path in paths {
        insert_path(&mut root, path);
    }

    print_tree_recursive(&root, "");
}

fn print_extensions(paths: &[&str]) {
    if paths.is_empty() {
        out!("   (нет файлов)");
        return;
    }

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for path in paths {
        let extension = extension_of(path)
            .unwrap_or_else(|| "<без расширения>".to_string());

        *counts.entry(extension).or_insert(0) += 1;
    }

    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|left, right| {
        right.1.cmp(&left.1).then(left.0.cmp(&right.0))
    });

    for (extension, count) in entries {
        if extension.starts_with('<') {
            out!("   {extension} — {count} файл(ов)");
        } else {
            out!("   .{extension:<10} — {count} файл(ов)");
        }
    }
}

pub fn print_captured_overview(paths: &[&str]) {
    out!("\n🌳 Захваченные файлы:");
    print_captured_tree(paths);
    out!("\n📄 Расширения захваченных файлов:");
    print_extensions(paths);
}

pub fn print_skipped(skipped: &[String], indent: &str, label: &str) {
    if skipped.is_empty() {
        return;
    }

    out!("{indent}{label}:");

    for path in skipped {
        out!("{indent}- {path}");
    }
}
