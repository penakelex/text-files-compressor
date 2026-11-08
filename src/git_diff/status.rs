use super::paths::unquote_git_path;

pub(crate) struct GitStatus {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

pub(crate) fn parse_status(output: &[u8]) -> GitStatus {
    let mut status = GitStatus {
        added: Vec::new(),
        modified: Vec::new(),
        deleted: Vec::new(),
    };

    for line in String::from_utf8_lossy(output).lines() {
        if line.len() < 4 {
            continue;
        }

        let code = &line[0..2];
        let raw_path = line[3..].trim();

        if code.contains(['R', 'C']) {
            if let Some((old_path, new_path)) =
                raw_path.split_once(" -> ")
            {
                status
                    .deleted
                    .push(unquote_git_path(old_path));
                status
                    .added
                    .push(unquote_git_path(new_path));
            }

            continue;
        }

        let path = unquote_git_path(raw_path);

        if code.contains(['?', 'A']) {
            status.added.push(path);
        } else if code.contains(['M', 'T']) {
            status.modified.push(path);
        } else if code.contains('D') {
            status.deleted.push(path);
        }
    }

    status
}
