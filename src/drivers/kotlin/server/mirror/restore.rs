//! Putting back the imports of a file that the server deleted without being
//! asked: what `keep_imports` decides, applied to the text of one file.

/// The imported name of an `import` line, without alias or wildcard.
pub(super) fn import_fqn(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("import")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = rest.split("//").next()?.trim().trim_end_matches(';');
    rest.split_whitespace().next()
}

/// `after`, plus every import line of `before` that it lost without
/// `accounted` calling for the loss. A restored line goes after the import
/// that stood before it, so the block keeps its order. A lost import that the
/// change calls for but `rewritten` gives a new name for (the server deleted
/// it as unused instead of rewriting it) comes back under that name.
pub(super) fn restore(
    before: &str,
    after: &str,
    accounted: &dyn Fn(&str) -> bool,
    rewritten: &dyn Fn(&str) -> Option<String>,
) -> String {
    let mut result: Vec<String> = after.split_inclusive('\n').map(str::to_string).collect();
    let mut anchor: Option<usize> = None;
    for line in before.split_inclusive('\n') {
        let Some(fqn) = import_fqn(line) else {
            continue;
        };
        let key = line.trim();
        if let Some(at) = result.iter().position(|kept| kept.trim() == key) {
            anchor = Some(at);
            continue;
        }
        let mut restored = if !accounted(fqn) {
            line.to_string()
        } else if let Some(new) = rewritten(fqn) {
            line.replacen(fqn, &new, 1)
        } else {
            continue;
        };
        // The rewritten import may be there already: the server did its part.
        if let Some(at) = result
            .iter()
            .position(|kept| kept.trim() == restored.trim())
        {
            anchor = Some(at);
            continue;
        }
        let at = anchor.map_or_else(|| first_import_place(&result), |anchor| anchor + 1);
        if !restored.ends_with('\n') {
            restored.push('\n');
        }
        result.insert(at, restored);
        anchor = Some(at);
    }
    result.concat()
}

/// Where the import block starts: at its first import, else after the
/// `package` line, else at the top.
fn first_import_place(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|line| import_fqn(line).is_some())
        .or_else(|| {
            lines
                .iter()
                .position(|line| line.trim_start().starts_with("package "))
                .map(|at| at + 1)
        })
        .unwrap_or(0)
}
