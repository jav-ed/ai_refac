pub(super) fn find_matching_bracket(bytes: &[u8], open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open_index;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    None
}

pub(super) fn parse_inline_destination(
    bytes: &[u8],
    open_paren: usize,
) -> Option<(usize, usize, usize)> {
    let mut index = open_paren + 1;
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }

    if index >= bytes.len() {
        return None;
    }

    if bytes[index] == b'<' {
        let href_start = index + 1;
        let mut cursor = href_start;
        while cursor < bytes.len() && bytes[cursor] != b'>' {
            if bytes[cursor] == b'\\' {
                cursor += 2;
            } else {
                cursor += 1;
            }
        }

        if bytes.get(cursor) != Some(&b'>') {
            return None;
        }

        let href_end = cursor;
        cursor += 1;
        while bytes
            .get(cursor)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            cursor += 1;
        }

        if bytes.get(cursor) != Some(&b')') {
            return None;
        }

        return Some((href_start, href_end, cursor));
    }

    let href_start = index;
    let mut href_end = None;
    let mut depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                if depth == 0 {
                    let resolved_end = href_end.unwrap_or(index);
                    return Some((href_start, resolved_end, index));
                }
                depth -= 1;
                index += 1;
            }
            byte if byte.is_ascii_whitespace() && depth == 0 => {
                href_end = Some(index);
                index += 1;

                while bytes.get(index).is_some_and(|ws| ws.is_ascii_whitespace()) {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    None
}

pub(super) fn parse_reference_definition_destination(
    bytes: &[u8],
    colon_index: usize,
    line_end: usize,
) -> Option<(usize, usize)> {
    let mut index = colon_index + 1;
    while index < line_end && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    if index >= line_end {
        return None;
    }

    if bytes[index] == b'<' {
        let href_start = index + 1;
        let mut cursor = href_start;
        while cursor < line_end && bytes[cursor] != b'>' {
            if bytes[cursor] == b'\\' {
                cursor += 2;
            } else {
                cursor += 1;
            }
        }

        if cursor >= line_end || bytes[cursor] != b'>' || cursor == href_start {
            return None;
        }

        return Some((href_start, cursor));
    }

    let href_start = index;
    while index < line_end && !bytes[index].is_ascii_whitespace() {
        if bytes[index] == b'\\' {
            index += 2;
        } else {
            index += 1;
        }
    }

    if href_start == index {
        None
    } else {
        Some((href_start, index))
    }
}

pub(super) fn normalize_reference_label(label: &str) -> String {
    let mut normalized = String::new();
    let mut pending_space = false;

    for ch in label.trim().chars() {
        if ch.is_whitespace() {
            pending_space = !normalized.is_empty();
            continue;
        }

        if pending_space {
            normalized.push(' ');
            pending_space = false;
        }

        normalized.extend(ch.to_lowercase());
    }

    normalized
}
