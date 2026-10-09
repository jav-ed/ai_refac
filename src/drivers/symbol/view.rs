//! The project's files as a dry-run batch sees them. A real batch writes each
//! rename before it plans the next, so the next one sees the files the one
//! before left. A dry run writes nothing, so it keeps what the earlier steps
//! would have written here and reads through it: `read_to_string` answers with
//! the pending text of a file when there is one and with the disk otherwise.
//!
//! The view belongs to one `scoped` future (a task-local value), so two dry
//! runs in one process, such as parallel tests, never see each other's files.

use anyhow::{Result, bail};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};

tokio::task_local! {
    static PENDING: RefCell<HashMap<PathBuf, String>>;
}

/// Runs `work` with an empty view: `remember` is allowed inside it.
pub async fn scoped<F: Future>(work: F) -> F::Output {
    PENDING.scope(RefCell::new(HashMap::new()), work).await
}

/// What a file reads as: the text an earlier step of the dry run would have
/// written, or else the file on disk. Positions are those of the text as read,
/// so a file with a byte order mark keeps it here as on disk.
pub fn read_to_string(path: &Path) -> io::Result<String> {
    let pending = PENDING
        .try_with(|files| files.borrow().get(path).cloned())
        .ok()
        .flatten();
    match pending {
        Some(text) => Ok(text),
        None => std::fs::read_to_string(path),
    }
}

/// A step of the dry run would have written `text` to `path`. Outside a
/// `scoped` view there is nothing to remember it in, which is a bug of the
/// caller and stops the command instead of being ignored.
pub fn remember(path: &Path, text: String) -> Result<()> {
    if PENDING
        .try_with(|files| files.borrow_mut().insert(path.to_path_buf(), text))
        .is_err()
    {
        bail!(
            "Internal error: a dry-run step tried to remember {} outside a dry-run view",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_remembered_text_hides_the_disk_only_inside_its_view() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "disk").unwrap();
        scoped(async {
            assert_eq!(read_to_string(&file).unwrap(), "disk");
            remember(&file, "pending".into()).unwrap();
            assert_eq!(read_to_string(&file).unwrap(), "pending");
        })
        .await;
        assert_eq!(read_to_string(&file).unwrap(), "disk");
    }

    #[tokio::test]
    async fn remembering_outside_a_view_is_an_error() {
        let error = remember(Path::new("/x"), String::new()).unwrap_err();
        assert!(format!("{error}").contains("outside a dry-run view"));
    }

    #[tokio::test]
    async fn two_views_do_not_share_files() {
        let file = PathBuf::from("/refac-view-test/never-on-disk.txt");
        let first = scoped(async {
            remember(&file, "one".into()).unwrap();
            read_to_string(&file).unwrap()
        });
        let second = scoped(async { read_to_string(&file).is_err() });
        let (first, second) = tokio::join!(first, second);
        assert_eq!(first, "one");
        assert!(second, "the second view must not see the first view's file");
    }
}
