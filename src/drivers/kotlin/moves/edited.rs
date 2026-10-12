//! The note that names the files whose content a Kotlin move changes, so a
//! reader sees at once what refac touches (and that a change anywhere else in
//! `git diff` is not from the move) without comparing the whole project. It is
//! worded in the present tense because a dry run prints the same notes.

use std::path::{Path, PathBuf};

/// A move of a folder can edit hundreds of files; the note names this many.
const SHOWN: usize = 40;

/// `None` when no file changed its content.
pub(super) fn note(root: &Path, edited: &[PathBuf]) -> Option<String> {
    let mut files: Vec<String> = edited
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    files.sort();
    files.dedup();
    if files.is_empty() {
        return None;
    }
    let count = files.len();
    let more = match count.checked_sub(SHOWN) {
        Some(rest) if rest > 0 => format!(" and {rest} more"),
        _ => String::new(),
    };
    files.truncate(SHOWN);
    Some(format!(
        "This move changes the content of {count} file{}: {}{more}. They are the moved files whose package line or imports change and the files that refer to them; a change in any other file is not from this move. `git diff --stat` shows the size of each.",
        if count == 1 { "" } else { "s" },
        files.join(", ")
    ))
}
