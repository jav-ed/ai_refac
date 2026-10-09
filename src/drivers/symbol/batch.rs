//! How a batch of renames reports a failure, for every engine that runs one:
//! which rename of the batch it was and what became of the renames before it.
//! A batch of one is a plain rename and keeps its messages as they are.

use super::rename::RenameRequest;

/// Name the rename of a batch an error belongs to.
pub fn in_batch(error: anyhow::Error, index: usize, requests: &[RenameRequest]) -> anyhow::Error {
    if requests.len() == 1 {
        return error;
    }
    error.context(label(index, requests))
}

pub fn label(index: usize, requests: &[RenameRequest]) -> String {
    let request = &requests[index];
    format!(
        "Rename {} of {} ({} -> {} in {})",
        index + 1,
        requests.len(),
        request.symbol,
        request.new_name,
        request.file.display()
    )
}

/// The error of the rename `failed`, after `applied` earlier renames were
/// written and an attempt was made to take them back; `undo_failures` are the
/// messages of the steps that could not be taken back. Reads outermost first:
/// which rename failed and what became of the batch, then why.
pub fn step_failed(
    error: anyhow::Error,
    failed: usize,
    requests: &[RenameRequest],
    applied: usize,
    undo_failures: Vec<String>,
) -> anyhow::Error {
    if requests.len() == 1 {
        return error;
    }
    let outcome = if !undo_failures.is_empty() {
        format!(
            "undoing the {applied} earlier rename(s) failed too, so the project is half refactored; inspect it with git:\n{}",
            undo_failures.join("\n")
        )
    } else if applied == 0 {
        "nothing was changed".to_string()
    } else {
        format!("the {applied} earlier rename(s) were undone, so nothing was changed")
    };
    error.context(format!("{} failed; {outcome}", label(failed, requests)))
}
