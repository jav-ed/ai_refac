//! A language server started for one rename: Go's gopls, rust-analyzer,
//! Pyright and the Dart server. It is started on demand, shown the file, asked
//! until it has loaded the project, and shut down when the rename is over, so
//! nothing stays resident. What differs per server (capabilities and the
//! readiness signal) lives in `lsp::client::Server`.

use super::server::RenameServer;
use crate::drivers::lsp::client::{self, Server};
use crate::drivers::lsp::rename::plan::discover::file_uri;
use crate::drivers::lsp::session::{LspSession, SessionConfig};
use anyhow::{Result, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How a server is started.
pub struct Launch<'a> {
    pub server: Server,
    pub executable: &'a Path,
    pub args: Vec<String>,
    /// Working directory of the process. Rustup picks the toolchain of the
    /// project from here, so it is the project root.
    pub cwd: &'a Path,
    pub root: &'a Path,
    /// Files opened before the readiness wait.
    pub documents: &'a [PathBuf],
}

pub struct ProjectServer {
    session: LspSession,
    server: Server,
    timeout: Duration,
    /// The text the server holds for each document shown to it.
    shown: HashMap<PathBuf, String>,
    /// Documents changed since the last `settle`.
    changed: Vec<PathBuf>,
}

impl ProjectServer {
    pub async fn start(launch: Launch<'_>) -> Result<Self> {
        let timeout = client::timeout()?;
        let mut capabilities = launch.server.capabilities(false);
        capabilities["textDocument"] = json!({ "rename": { "prepareSupport": true } });
        let (session, init) = LspSession::start(SessionConfig {
            name: launch.server.name(),
            executable: launch.executable,
            args: launch.args,
            cwd: launch.cwd,
            root: launch.root,
            capabilities,
            keep_notifications: launch.server.kept_notifications(),
            language_id: launch.server.language_id(),
            env: Vec::new(),
            initialization_options: None,
        })
        .await?;
        let provided = &init["capabilities"];
        let missing = launch.server.missing_capabilities(provided);
        if !missing.is_empty() {
            session.shutdown().await;
            bail!(
                "The {} at {} does not offer {}; it is too old, the wrong program, or not the server refac needs for this language (`refac doctor` names it)",
                launch.server.name(),
                launch.executable.display(),
                missing.join(", ")
            );
        }
        let mut started = Self {
            session,
            server: launch.server,
            timeout,
            shown: HashMap::new(),
            changed: Vec::new(),
        };
        match started.load(launch.documents).await {
            Ok(()) => Ok(started),
            Err(error) => {
                let name = started.server.name();
                started.session.shutdown().await;
                Err(error.context(format!("The {name} did not finish loading the project")))
            }
        }
    }

    async fn load(&mut self, documents: &[PathBuf]) -> Result<()> {
        self.session.initialized().await?;
        for path in documents {
            let text = tokio::fs::read_to_string(path).await?;
            self.sync_document(path, &text).await?;
        }
        self.server
            .wait_ready(&mut self.session, self.timeout, documents)
            .await?;
        // Everything shown so far is part of the loaded project.
        self.changed.clear();
        Ok(())
    }
}

#[async_trait]
impl RenameServer for ProjectServer {
    /// A server that is still working answers "content modified"; that is the
    /// one error worth repeating, a bounded number of times.
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        for attempt in 1..=client::RENAME_ATTEMPTS {
            let answer =
                tokio::time::timeout(self.timeout, self.session.request(method, params.clone()))
                    .await
                    .map_err(|_| {
                        anyhow::anyhow!(
                            "The {} did not answer {method} within {} seconds (REFAC_LSP_TIMEOUT_SECS changes the limit)",
                            self.server.name(),
                            self.timeout.as_secs()
                        )
                    })?;
            match answer {
                Err(error)
                    if attempt < client::RENAME_ATTEMPTS && client::is_content_modified(&error) =>
                {
                    tokio::time::sleep(client::CONTENT_MODIFIED_PAUSE).await;
                }
                other => return other,
            }
        }
        unreachable!("the last attempt always returns")
    }

    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        if self.shown.get(path).map(String::as_str) != Some(text) {
            self.shown.insert(path.to_path_buf(), text.to_string());
            self.changed.push(path.to_path_buf());
        }
        self.session.sync_document(path, text).await
    }

    async fn settle(&mut self) -> Result<()> {
        let changed = std::mem::take(&mut self.changed);
        let Some(barrier) = self.server.analysis_barrier() else {
            return Ok(());
        };
        for path in changed {
            let params = json!({ "textDocument": { "uri": file_uri(&path)? } });
            self.request(barrier, params).await?;
        }
        Ok(())
    }

    async fn shutdown(self: Box<Self>) {
        self.session.shutdown().await;
    }
}
