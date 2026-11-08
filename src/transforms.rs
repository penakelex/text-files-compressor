use std::borrow::Cow;

use crate::config::{EolMode, INDENT_WIDTH, RLE_CHAR};

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

fn indent_rle(line: &str) -> Cow<'_, str> {
    let indent_length =
        line.len() - line.trim_start_matches(' ').len();

    if indent_length < INDENT_WIDTH {
        return Cow::Borrowed(line);
    }

    let levels = indent_length / INDENT_WIDTH;
    let remainder = indent_length % INDENT_WIDTH;
    let mut result = String::with_capacity(
        line.len() - indent_length + levels + remainder,
    );

    for _ in 0..levels {
        result.push(RLE_CHAR);
    }

    result.push_str(&" ".repeat(remainder));
    result.push_str(&line[indent_length..]);

    Cow::Owned(result)
}

pub fn transform_content(
    content: &str,
    eol_mode: EolMode,
    indent_rle_enabled: bool,
    trim_trailing: bool,
    collapse_blank_lines: Option<usize>,
) -> String {
    let mut result = String::with_capacity(content.len());
    let mut blank_run = 0usize;

    for line in content.split_inclusive('\n') {
        let (body, ending) = split_line_ending(line);
        let is_blank = body.trim().is_empty();

        if is_blank {
            blank_run += 1;

            if let Some(limit) = collapse_blank_lines
                && blank_run > limit
            {
                continue;
            }
        } else {
            blank_run = 0;
        }

        let body_output = if trim_trailing {
            Cow::Borrowed(body.trim_end_matches([' ', '\t']))
        } else {
            Cow::Borrowed(body)
        };
        let body_output = if !is_blank && indent_rle_enabled {
            indent_rle(&body_output)
        } else {
            body_output
        };

        result.push_str(&body_output);

        match eol_mode {
            EolMode::Auto => result.push_str(ending),
            EolMode::Lf => {
                if !ending.is_empty() {
                    result.push('\n');
                }
            }
            EolMode::Crlf => {
                if !ending.is_empty() {
                    result.push_str("\r\n");
                }
            }
        }
    }

    result
}
