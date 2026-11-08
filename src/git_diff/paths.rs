pub(crate) fn unquote_git_path(path: &str) -> String {
    if path.len() >= 2
        && path.starts_with('"')
        && path.ends_with('"')
    {
        path[1..path.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        path.to_string()
    }
}
