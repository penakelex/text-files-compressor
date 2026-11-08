#[derive(Default)]
pub(crate) struct GitStatus {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub non_utf8_paths: Vec<String>,
}

pub(crate) fn parse_status(output: &[u8]) -> GitStatus {
    let mut status = GitStatus::default();

    let parts: Vec<&[u8]> = output.split(|&byte| byte == 0).collect();

    let mut index = 0;
    while index < parts.len() {
        let part = parts[index];

        if part.is_empty() || part.len() < 4 {
            index += 1;
            continue;
        }

        let code = &part[0..2];
        let raw_path = &part[3..];

        let is_rename = code.contains(&b'R');
        let is_copy = code.contains(&b'C');

        if is_rename || is_copy {
            let new_ok = std::str::from_utf8(raw_path).ok();
            let old_bytes = parts.get(index + 1).copied();
            let old_ok = old_bytes
                .and_then(|bytes| std::str::from_utf8(bytes).ok());

            if let Some(new_path) = new_ok {
                if is_rename
                    && let Some(old) = old_ok
                    && !old.is_empty()
                {
                    status.deleted.push(old.to_string());
                }
                status.added.push(new_path.to_string());
            } else {
                status.non_utf8_paths.push(
                    String::from_utf8_lossy(raw_path).into_owned(),
                );
            }

            if let Some(old_bytes) = old_bytes
                && old_ok.is_none()
            {
                status.non_utf8_paths.push(
                    String::from_utf8_lossy(old_bytes).into_owned(),
                );
            }

            index += 2;
            continue;
        }

        let raw_str = match std::str::from_utf8(raw_path) {
            Ok(value) => value,
            Err(_) => {
                status.non_utf8_paths.push(
                    String::from_utf8_lossy(raw_path).into_owned(),
                );
                index += 1;
                continue;
            }
        };

        let path = raw_str.to_string();
        index += 1;

        if path.ends_with('/') {
            continue;
        }

        let first = code[0];
        let second = code[1];

        let is_unmerged = matches!(
            (first, second),
            (b'D', b'D')
                | (b'A', b'U')
                | (b'U', b'D')
                | (b'U', b'A')
                | (b'D', b'U')
                | (b'A', b'A')
                | (b'U', b'U')
        );

        if is_unmerged {
            status.modified.push(path);
        } else if code.contains(&b'?') || code.contains(&b'A') {
            status.added.push(path);
        } else if code.contains(&b'M') || code.contains(&b'T') {
            status.modified.push(path);
        } else if code.contains(&b'D') {
            status.deleted.push(path);
        }
    }

    status
}
