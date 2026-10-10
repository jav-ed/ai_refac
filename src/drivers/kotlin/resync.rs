//! Telling the Kotlin server what a rename wrote, when the next rename of a
//! batch runs in the same session. The server keeps its own view of the
//! project: after a class rename moved a file it still lists the old path, and
//! it never reads a file it was shown unless told that the file changed. The
//! other languages' servers only hold the texts they were shown, which are the
//! texts that were written.

use super::server::KotlinServer;
use crate::drivers::lsp::rename::plan::discover::file_uri;
use crate::drivers::lsp::session::RpcError;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// `FileChangeType` of `workspace/didChangeWatchedFiles`.
const CREATED: u8 = 1;
const CHANGED: u8 = 2;
const DELETED: u8 = 3;

/// The file events of a rename: every moved file is gone from its old path and
/// new at its new one, and every edited file, at the path it has now, changed.
fn changes(edited: &[PathBuf], moves: &[(PathBuf, PathBuf)]) -> Result<Vec<Value>> {
    let mut changes = Vec::new();
    for (from, to) in moves {
        changes.push(json!({ "uri": file_uri(from)?, "type": DELETED }));
        changes.push(json!({ "uri": file_uri(to)?, "type": CREATED }));
    }
    for path in now_at(edited, moves) {
        changes.push(json!({ "uri": file_uri(&path)?, "type": CHANGED }));
    }
    Ok(changes)
}

/// Where the edited files are after the moves.
fn now_at(edited: &[PathBuf], moves: &[(PathBuf, PathBuf)]) -> Vec<PathBuf> {
    edited
        .iter()
        .map(|path| {
            moves
                .iter()
                .find(|(from, _)| from == path)
                .map_or_else(|| path.clone(), |(_, to)| to.clone())
        })
        .collect()
}

/// The watcher event alone is handled by the server asynchronously, so a
/// request sent right after it could still see the old project. Documents sent
/// with their text are known the moment the notification is read, in order,
/// before the next request.
pub async fn after_rename(
    server: &mut KotlinServer,
    edited: &[PathBuf],
    moves: &[(PathBuf, PathBuf)],
) -> Result<()> {
    for (from, _) in moves {
        server.close_under(from).await?;
    }
    let changes = changes(edited, moves)?;
    server
        .notify(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": changes }),
        )
        .await?;
    for path in now_at(edited, moves) {
        if !is_source(&path) {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        server.sync_document(&path, &text).await?;
    }
    Ok(())
}

/// What changed on disk without the server being told, for a caller that
/// keeps one server across operations and edits or restores files itself.
#[derive(Debug, Default)]
pub struct DiskChanges {
    pub created: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

impl DiskChanges {
    pub fn is_empty(&self) -> bool {
        self.created.is_empty() && self.changed.is_empty() && self.deleted.is_empty()
    }
}

/// How long the server may take to drop a deleted file. Measured: 0.2 seconds.
const DELETION_WAIT: Duration = Duration::from_secs(10);

/// Tell the server the files changed behind its back. Every open document
/// under the project is closed first, because a document the server holds is
/// the text it was shown, not the file; the sources that exist are then sent
/// with their text so that the next request already sees them (the watcher
/// event alone is handled later). A deleted source has no text to send, so
/// the call waits until the server no longer lists its symbols: a request
/// sent before that (a move into the folder it was in) is refused because the
/// file "already exists".
pub async fn follow_disk(
    server: &mut KotlinServer,
    root: &Path,
    changes: &DiskChanges,
) -> Result<()> {
    server.close_under(root).await?;
    let mut events = Vec::new();
    for (paths, kind) in [
        (&changes.created, CREATED),
        (&changes.changed, CHANGED),
        (&changes.deleted, DELETED),
    ] {
        for path in paths {
            events.push(json!({ "uri": file_uri(path)?, "type": kind }));
        }
    }
    if !events.is_empty() {
        server
            .notify(
                "workspace/didChangeWatchedFiles",
                json!({ "changes": events }),
            )
            .await?;
    }
    for path in changes.created.iter().chain(&changes.changed) {
        if !is_source(path) {
            continue;
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        server.sync_document(path, &text).await?;
    }
    for path in changes.deleted.iter().filter(|path| is_source(path)) {
        wait_until_dropped(server, path).await?;
    }
    Ok(())
}

/// Asks for the symbols of the file until the server has none (or no longer
/// knows the file).
async fn wait_until_dropped(server: &mut KotlinServer, path: &Path) -> Result<()> {
    let params = json!({ "textDocument": { "uri": file_uri(path)? } });
    let deadline = Instant::now() + DELETION_WAIT;
    loop {
        match server
            .request("textDocument/documentSymbol", params.clone())
            .await
        {
            Ok(symbols) if symbols.as_array().is_none_or(Vec::is_empty) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.downcast_ref::<RpcError>().is_some() => return Ok(()),
            Err(error) => return Err(error),
        }
        if Instant::now() > deadline {
            bail!(
                "The Kotlin server still lists the symbols of {} {} seconds after it was told the file is gone",
                path.display(),
                DELETION_WAIT.as_secs()
            );
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn is_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("kt" | "java")
    )
}

#[cfg(test)]
mod tests;
