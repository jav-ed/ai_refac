//! Whether one Gradle module names another as a project dependency, as far as
//! its build file says. A name declared in a module that is not a dependency
//! cannot be imported (the import would not resolve), so a move may only
//! assume the modules it can see written down.

use std::path::Path;

/// `module`'s build file mentions `other` as `project(":a:b")` or as the
/// type-safe accessor `projects.a.b`. Direct dependencies only: a module
/// reached through another module's `api` is not looked for. A build file
/// that cannot be read names nothing.
pub(super) fn names(root: &Path, module: &Path, other: &Path) -> bool {
    let Ok(relative) = other.strip_prefix(root) else {
        return false;
    };
    let segments: Vec<&str> = relative
        .components()
        .filter_map(|part| part.as_os_str().to_str())
        .collect();
    if segments.is_empty() {
        return false;
    }
    let Some(text) = ["build.gradle.kts", "build.gradle"]
        .iter()
        .find_map(|name| std::fs::read_to_string(module.join(name)).ok())
    else {
        return false;
    };
    let path = format!(":{}", segments.join(":"));
    let accessor = format!(
        "projects.{}",
        segments
            .iter()
            .map(|part| camel(part))
            .collect::<Vec<_>>()
            .join(".")
    );
    text.contains(&format!("\"{path}\""))
        || text.contains(&format!("'{path}'"))
        || mentions(&text, &accessor)
}

/// `feature-x` is `featureX` in an accessor.
fn camel(segment: &str) -> String {
    let mut name = String::new();
    let mut upper = false;
    for ch in segment.chars() {
        if matches!(ch, '-' | '_' | '.') {
            upper = true;
        } else if upper {
            name.extend(ch.to_uppercase());
            upper = false;
        } else {
            name.push(ch);
        }
    }
    name
}

/// `projects.lib` as a whole word, not the start of `projects.library`.
fn mentions(text: &str, accessor: &str) -> bool {
    text.match_indices(accessor).any(|(at, found)| {
        let before = text[..at].chars().next_back();
        let after = text[at + found.len()..].chars().next();
        !before.is_some_and(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')
            && !after.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
    })
}
