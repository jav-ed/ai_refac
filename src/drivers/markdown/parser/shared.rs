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

/// Returns true if `pos` falls inside any of the suppressed byte ranges.
pub(super) fn is_suppressed(pos: usize, ranges: &[std::ops::Range<usize>]) -> bool {
    ranges.iter().any(|r| r.contains(&pos))
}

/// Advances `pos` past the current line (including the `\n` if present).
fn advance_line(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < bytes.len() { pos + 1 } else { pos }
}

/// Computes byte ranges that should be skipped during link parsing:
/// fenced code blocks (``` or ~~~) and inline code spans (backtick pairs).
pub(super) fn compute_suppressed_ranges(content: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = content.as_bytes();
    let mut suppressed: Vec<std::ops::Range<usize>> = Vec::new();

    // Pass 1: fenced code blocks (scanned line-by-line).
    let mut i = 0usize;
    while i < bytes.len() {
        let line_start = i;

        // Optional leading indent: up to 3 spaces (CommonMark rule).
        let mut col = i;
        while col < bytes.len() && col - i < 3 && bytes[col] == b' ' {
            col += 1;
        }

        let fc = match bytes.get(col) {
            Some(&b'`') => b'`',
            Some(&b'~') => b'~',
            _ => {
                i = advance_line(bytes, i);
                continue;
            }
        };

        let fence_run_start = col;
        while col < bytes.len() && bytes[col] == fc {
            col += 1;
        }
        let fence_len = col - fence_run_start;

        if fence_len < 3 {
            i = advance_line(bytes, i);
            continue;
        }

        // Backtick fences: the info string must not contain a backtick.
        if fc == b'`' {
            let mut check = col;
            let mut valid = true;
            while check < bytes.len() && bytes[check] != b'\n' {
                if bytes[check] == b'`' {
                    valid = false;
                    break;
                }
                check += 1;
            }
            if !valid {
                i = advance_line(bytes, i);
                continue;
            }
        }

        // Advance past the opening fence line.
        i = advance_line(bytes, i);

        // Search for the closing fence.
        let mut j = i;
        let mut closed = false;
        while j < bytes.len() {
            let cline_start = j;
            let mut ccol = j;
            while ccol < bytes.len() && ccol - j < 3 && bytes[ccol] == b' ' {
                ccol += 1;
            }

            if bytes.get(ccol) == Some(&fc) {
                let close_run_start = ccol;
                while ccol < bytes.len() && bytes[ccol] == fc {
                    ccol += 1;
                }
                let close_len = ccol - close_run_start;

                if close_len >= fence_len {
                    let mut rest = ccol;
                    let mut valid_close = true;
                    while rest < bytes.len() && bytes[rest] != b'\n' {
                        if !bytes[rest].is_ascii_whitespace() {
                            valid_close = false;
                            break;
                        }
                        rest += 1;
                    }

                    if valid_close {
                        let close_end = advance_line(bytes, cline_start);
                        suppressed.push(line_start..close_end);
                        i = close_end;
                        closed = true;
                        break;
                    }
                }
            }

            j = advance_line(bytes, cline_start);
        }

        if !closed {
            suppressed.push(line_start..bytes.len());
            i = bytes.len();
        }
    }

    // Pass 2: inline code spans, skipping already-suppressed regions.
    let mut i = 0usize;
    while i < bytes.len() {
        if is_suppressed(i, &suppressed) {
            i += 1;
            continue;
        }

        if bytes[i] != b'`' {
            i += 1;
            continue;
        }

        let span_start = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let tick_len = i - span_start;

        // Search for the matching closing run of exactly tick_len backticks.
        let mut j = i;
        while j < bytes.len() {
            if is_suppressed(j, &suppressed) {
                j += 1;
                continue;
            }

            if bytes[j] == b'`' {
                let close_start = j;
                while j < bytes.len() && bytes[j] == b'`' {
                    j += 1;
                }
                if j - close_start == tick_len {
                    suppressed.push(span_start..j);
                    i = j;
                    break;
                }
                // Wrong run length — keep searching from j.
            } else {
                j += 1;
            }
        }
        // No close found: i already advanced past the opening ticks.
    }

    suppressed
}

#[cfg(test)]
mod tests {
    use super::{compute_suppressed_ranges, is_suppressed};

    #[test]
    fn suppresses_fenced_code_block() {
        let content = "before\n```\n[link](./path.md)\n```\nafter";
        let ranges = compute_suppressed_ranges(content);
        let fence_start = content.find("```").unwrap();
        let link_pos = content.find('[').unwrap();
        let after_pos = content.rfind("after").unwrap();
        assert!(is_suppressed(fence_start, &ranges), "opening fence line should be suppressed");
        assert!(is_suppressed(link_pos, &ranges), "link inside code block should be suppressed");
        assert!(!is_suppressed(0, &ranges), "content before fence should not be suppressed");
        assert!(!is_suppressed(after_pos, &ranges), "content after fence should not be suppressed");
    }

    #[test]
    fn suppresses_tilde_fenced_code_block() {
        let content = "~~~\n[link](./path.md)\n~~~\n";
        let ranges = compute_suppressed_ranges(content);
        let link_pos = content.find('[').unwrap();
        assert!(is_suppressed(link_pos, &ranges));
    }

    #[test]
    fn suppresses_inline_code_span() {
        let content = "See `[link](./path.md)` here.";
        let ranges = compute_suppressed_ranges(content);
        let link_pos = content.find('[').unwrap();
        assert!(is_suppressed(link_pos, &ranges), "link in backtick span should be suppressed");
        assert!(!is_suppressed(0, &ranges), "content outside span should not be suppressed");
    }

    #[test]
    fn does_not_suppress_normal_links() {
        let content = "[link](./path.md)";
        let ranges = compute_suppressed_ranges(content);
        assert!(ranges.is_empty());
    }

    #[test]
    fn suppresses_unclosed_fence_to_end_of_content() {
        let content = "```\n[link](./path.md)\n";
        let ranges = compute_suppressed_ranges(content);
        let link_pos = content.find('[').unwrap();
        assert!(is_suppressed(link_pos, &ranges));
    }
}
