//! A language server driven for one batch of file moves or symbol renames:
//! start it, show it the project, wait until it says it has loaded it, ask,
//! and apply the edits it answers. Used for Dart, Go, and Rust. The wire work
//! is the shared `LspSession`; `server.rs` holds what differs per server.
//! Nothing here sleeps for a fixed time: readiness is the server's own signal
//! and a missing signal is an error after `REFAC_LSP_TIMEOUT_SECS` (default
//! 300).

mod changes;
mod documents;
mod plan;
mod resource_ops;
mod server;

use crate::drivers::lsp::session::{LspSession, RpcError, SessionConfig};
use anyhow::{Context, Result, bail};
pub use changes::{PendingChange, PlanSummary, apply_pending_changes, summarize};
use changes::{apply_workspace_edit, collect_pending_changes};
pub use documents::collect_workspace_documents;
use lsp_types::WorkspaceEdit;
use serde_json::{Value, json};
pub use server::Server;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

const TIMEOUT_ENV: &str = "REFAC_LSP_TIMEOUT_SECS";
const DEFAULT_TIMEOUT_SECS: u64 = 300;
/// How often a rename is repeated when the server answers "content modified".
pub(crate) const RENAME_ATTEMPTS: u32 = 5;
pub(crate) const CONTENT_MODIFIED_PAUSE: Duration = Duration::from_secs(1);
const CONTENT_MODIFIED_CODE: i64 = -32801;

pub struct LspClient {
    binary_path: String,
}

/// A single symbol rename to be processed in a batch LSP session.
pub struct SymbolRenameRequest {
    pub document_path: std::path::PathBuf,
    pub position: lsp_types::Position,
    pub new_name: String,
    pub pending_moves: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
}

struct Started {
    session: LspSession,
    server: Server,
    root_dir: PathBuf,
    timeout: Duration,
}

impl LspClient {
    pub fn new(binary_path: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
        }
    }

    pub async fn check_availability(&self) -> Result<bool> {
        Ok(std::path::Path::new(&self.binary_path).exists())
    }

    /// Ask the server which edits moving the files takes, without writing
    /// anything, so the caller can check the plan before `apply_pending_changes`.
    /// An empty plan means the server answered that nothing has to change.
    pub async fn plan_file_renames(
        &self,
        args: &[&str],
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
        language_id: &str,
        file_extensions: &[&str],
    ) -> Result<Vec<PendingChange>> {
        let mut started = self.start(args, root_path, language_id, true).await?;
        let outcome = plan_in_session(&mut started, file_map, file_extensions).await;
        started.session.shutdown().await;
        outcome
    }

    pub async fn initialize_and_rename_files(
        &self,
        args: &[&str],
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
        language_id: &str,
        file_extensions: &[&str],
    ) -> Result<()> {
        let changes = self
            .plan_file_renames(args, file_map, root_path, language_id, file_extensions)
            .await?;
        apply_pending_changes(changes).await?;
        Ok(())
    }

    pub async fn initialize_and_rename_symbols_batch(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        renames: Vec<SymbolRenameRequest>,
        language_id: &str,
    ) -> Result<Vec<Vec<(PathBuf, PathBuf)>>> {
        if renames.is_empty() {
            return Ok(Vec::new());
        }
        let mut started = self.start(args, root_path, language_id, false).await?;
        let outcome = rename_in_session(&mut started, renames).await;
        started.session.shutdown().await;
        outcome
    }

    async fn start(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        language_id: &str,
        file_operations: bool,
    ) -> Result<Started> {
        let server = Server::for_language(language_id)?;
        let root_dir = match root_path {
            Some(root) => std::path::absolute(root)?,
            None => std::env::current_dir()?,
        };
        // The server starts in the directory refac runs in, as before: rustup
        // picks the rust-analyzer of the toolchain that directory pins.
        let cwd = std::env::current_dir()?;
        let executable = PathBuf::from(&self.binary_path);
        let (mut session, _) = LspSession::start(SessionConfig {
            name: server.name(),
            executable: &executable,
            args: args.iter().map(|arg| arg.to_string()).collect(),
            cwd: &cwd,
            root: &root_dir,
            capabilities: server.capabilities(file_operations),
            keep_notifications: server.kept_notifications(),
            language_id: server.language_id(),
            env: Vec::new(),
        })
        .await?;
        session.initialized().await?;
        Ok(Started {
            session,
            server,
            root_dir,
            timeout: timeout()?,
        })
    }
}

pub fn timeout() -> Result<Duration> {
    match std::env::var(TIMEOUT_ENV) {
        Err(_) => Ok(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
        Ok(value) => match value.trim().parse::<u64>() {
            Ok(seconds) if seconds > 0 => Ok(Duration::from_secs(seconds)),
            _ => bail!("{TIMEOUT_ENV} must be a positive number of seconds, got `{value}`"),
        },
    }
}

async fn plan_in_session(
    started: &mut Started,
    file_map: Vec<(String, String)>,
    file_extensions: &[&str],
) -> Result<Vec<PendingChange>> {
    let documents = collect_workspace_documents(&started.root_dir, file_extensions)?;
    for path in &documents {
        let text = tokio::fs::read_to_string(path).await?;
        started.session.sync_document(path, &text).await?;
    }
    started
        .server
        .wait_ready(&mut started.session, started.timeout, &documents)
        .await?;

    let mut files = Vec::new();
    // Build a reverse map: new_abs_path → old_abs_path.
    // When the LSP returns a workspace edit targeting a file at its
    // *new* (not-yet-existing) path, we redirect the edit to the
    // source path so it is applied before the filesystem move.
    let mut pending_moves: HashMap<PathBuf, PathBuf> = HashMap::new();
    for (source, target) in file_map {
        let source_abs = resolve_abs_path(&started.root_dir, Path::new(&source));
        let target_abs = resolve_abs_path(&started.root_dir, Path::new(&target));
        files.push(json!({ "oldUri": file_uri(&source_abs)?, "newUri": file_uri(&target_abs)? }));
        pending_moves.insert(target_abs, source_abs);
    }

    let answer = ask(
        started,
        "workspace/willRenameFiles",
        json!({ "files": files }),
    )
    .await?;
    tracing::debug!("LSP willRenameFiles response: {:?}", answer);
    match workspace_edit(answer, "workspace/willRenameFiles")? {
        Some(edit) => collect_pending_changes(edit, &pending_moves),
        None => Ok(Vec::new()),
    }
}

async fn rename_in_session(
    started: &mut Started,
    renames: Vec<SymbolRenameRequest>,
) -> Result<Vec<Vec<(PathBuf, PathBuf)>>> {
    show_documents(started, &renames).await?;

    let mut all_file_renames = Vec::new();
    for rename in renames {
        let edit = rename_edit(started, &rename).await?;
        let (file_renames, modified_paths) =
            apply_workspace_edit(edit, &rename.pending_moves).await?;

        // Notify the LSP of every file we just changed on disk so
        // subsequent renames in this session see the updated state. A file the
        // same edit also moved is shown at its new place and closed at the old.
        for path in modified_paths {
            let current = relocated(&path, &file_renames);
            let content = tokio::fs::read_to_string(&current)
                .await
                .with_context(|| format!("Cannot read {} after editing it", current.display()))?;
            if current != path {
                started.session.close_under(&path).await?;
            }
            started.session.sync_document(&current, &content).await?;
        }
        all_file_renames.push(file_renames);
    }
    Ok(all_file_renames)
}

/// Opens every document the renames start from, so the server has them in its
/// view before the first request, and waits until it has loaded the project.
async fn show_documents(started: &mut Started, renames: &[SymbolRenameRequest]) -> Result<()> {
    let mut opened: Vec<PathBuf> = Vec::new();
    for rename in renames {
        let path = resolve_abs_path(&started.root_dir, &rename.document_path);
        if !opened.contains(&path) {
            let text = tokio::fs::read_to_string(&path)
                .await
                .with_context(|| format!("Cannot read {}", path.display()))?;
            started.session.sync_document(&path, &text).await?;
            opened.push(path);
        }
    }
    started
        .server
        .wait_ready(&mut started.session, started.timeout, &opened)
        .await
}

/// The edit the server proposes for one rename.
async fn rename_edit(started: &mut Started, rename: &SymbolRenameRequest) -> Result<WorkspaceEdit> {
    let document = resolve_abs_path(&started.root_dir, &rename.document_path);
    let params = json!({
        "textDocument": { "uri": file_uri(&document)? },
        "position": rename.position,
        "newName": rename.new_name,
    });
    let answer = rename_with_retries(started, params).await?;
    let Some(edit) = workspace_edit(answer, "textDocument/rename")? else {
        bail!("textDocument/rename returned no workspace edit");
    };
    Ok(edit)
}

/// A server that is still working answers "content modified"; that is the one
/// error worth repeating, a bounded number of times.
async fn rename_with_retries(started: &mut Started, params: Value) -> Result<Value> {
    for attempt in 1..=RENAME_ATTEMPTS {
        match request(started, "textDocument/rename", params.clone()).await {
            Err(error) if attempt < RENAME_ATTEMPTS && is_content_modified(&error) => {
                tokio::time::sleep(CONTENT_MODIFIED_PAUSE).await;
            }
            other => return other.map_err(|error| describe("textDocument/rename", error)),
        }
    }
    unreachable!("the last attempt always returns")
}

pub(crate) fn is_content_modified(error: &anyhow::Error) -> bool {
    error.downcast_ref::<RpcError>().is_some_and(|rpc| {
        rpc.code == CONTENT_MODIFIED_CODE || rpc.message.eq_ignore_ascii_case("content modified")
    })
}

async fn request(started: &mut Started, method: &str, params: Value) -> Result<Value> {
    tokio::time::timeout(started.timeout, started.session.request(method, params))
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "The {} did not answer {method} within {} seconds ({TIMEOUT_ENV} changes the limit)",
                started.server.name(),
                started.timeout.as_secs()
            )
        })?
}

async fn ask(started: &mut Started, method: &str, params: Value) -> Result<Value> {
    request(started, method, params)
        .await
        .map_err(|error| describe(method, error))
}

/// An error answer reads `method failed (code): message`, which is what the
/// caller prints.
fn describe(method: &str, error: anyhow::Error) -> anyhow::Error {
    match error.downcast_ref::<RpcError>() {
        Some(rpc) => anyhow::anyhow!("{method} failed ({}): {}", rpc.code, rpc.message),
        None => error,
    }
}

fn workspace_edit(answer: Value, method: &str) -> Result<Option<WorkspaceEdit>> {
    if answer.is_null() {
        return Ok(None);
    }
    serde_json::from_value(answer)
        .map(Some)
        .with_context(|| format!("Failed to parse {method} response as WorkspaceEdit"))
}

fn file_uri(path: &Path) -> Result<String> {
    Url::from_file_path(path)
        .map(|url| url.to_string())
        .map_err(|_| anyhow::anyhow!("Invalid path {}", path.display()))
}

/// Where a path is after the file renames of one workspace edit (a renamed
/// directory carries what is below it).
fn relocated(path: &Path, renames: &[(PathBuf, PathBuf)]) -> PathBuf {
    renames
        .iter()
        .find_map(|(old, new)| {
            let rest = path.strip_prefix(old).ok()?;
            // `join("")` would add a trailing slash.
            Some(if rest.as_os_str().is_empty() {
                new.clone()
            } else {
                new.join(rest)
            })
        })
        .unwrap_or_else(|| path.to_path_buf())
}

fn resolve_abs_path(root_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    }
}

#[cfg(test)]
mod tests;
