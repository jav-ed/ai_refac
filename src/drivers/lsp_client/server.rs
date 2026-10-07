//! What differs between the language servers behind `LspClient`: their
//! capabilities, and the signal that says the project is loaded and a request
//! will be answered from the real project instead of an empty one. Each signal
//! below was observed on the real server; none of them is a sleep.

use crate::drivers::lsp_session::LspSession;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Server {
    Dart,
    Go,
    Rust,
    Pyrefly,
}

impl Server {
    pub fn for_language(language_id: &str) -> Result<Self> {
        match language_id {
            "dart" => Ok(Self::Dart),
            "go" => Ok(Self::Go),
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Pyrefly),
            other => bail!("No language server profile for `{other}`"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dart => "Dart analysis server",
            Self::Go => "gopls",
            Self::Rust => "rust-analyzer",
            Self::Pyrefly => "Pyrefly",
        }
    }

    pub fn language_id(self) -> fn(&Path) -> &'static str {
        match self {
            Self::Dart => |_| "dart",
            Self::Go => |_| "go",
            Self::Rust => |_| "rust",
            Self::Pyrefly => |_| "python",
        }
    }

    /// The notifications the readiness signal is made of.
    pub fn kept_notifications(self) -> &'static [&'static str] {
        match self {
            Self::Dart => &["$/analyzerStatus"],
            Self::Go => &["$/progress"],
            Self::Rust => &["experimental/serverStatus"],
            Self::Pyrefly => &["textDocument/publishDiagnostics"],
        }
    }

    pub fn capabilities(self, file_operations: bool) -> Value {
        let mut workspace = json!({
            "applyEdit": true,
            "workspaceEdit": {
                "documentChanges": true,
                "resourceOperations": ["create", "rename", "delete"],
                "failureHandling": "transactional",
                "normalizesLineEndings": false,
            },
            "workspaceFolders": true,
        });
        if file_operations {
            workspace["fileOperations"] = json!({ "willRename": true });
        }
        let mut capabilities = json!({ "workspace": workspace });
        match self {
            // gopls reports its workspace load as work-done progress, and only
            // to a client that says it can show one.
            Self::Go => capabilities["window"] = json!({ "workDoneProgress": true }),
            Self::Rust => {
                capabilities["experimental"] = json!({ "serverStatusNotification": true })
            }
            // Without workDoneProgress the Dart server sends `$/analyzerStatus`.
            Self::Dart | Self::Pyrefly => {}
        }
        capabilities
    }

    /// Wait until the server has loaded the project, or fail loudly.
    /// `documents` are the files the session has opened.
    pub async fn wait_ready(
        self,
        session: &mut LspSession,
        timeout: Duration,
        documents: &[PathBuf],
    ) -> Result<()> {
        match self {
            Self::Dart => wait_for_analysis(session, timeout).await,
            // "Setting up workspace" begins and ends once the packages are loaded.
            Self::Go => session
                .wait_notification(|note| note["params"]["value"]["kind"] == "end", timeout)
                .await
                .map(|_| ()),
            // `quiescent` is true when loading and indexing are over.
            Self::Rust => session
                .wait_notification(|note| note["params"]["quiescent"] == true, timeout)
                .await
                .map(|_| ()),
            // Pyrefly announces nothing but diagnostics; it has checked a file
            // once it has published them.
            Self::Pyrefly => {
                for document in documents {
                    let uri = Url::from_file_path(document)
                        .map_err(|_| anyhow::anyhow!("Invalid path {}", document.display()))?
                        .to_string();
                    session
                        .wait_notification(|note| note["params"]["uri"] == uri.as_str(), timeout)
                        .await?;
                }
                Ok(())
            }
        }
    }
}

/// The Dart server reports `isAnalyzing` true when it starts and false when it
/// is done. Documents opened in one burst can start more than one cycle, so
/// readiness is a finished cycle with nothing newer already waiting.
async fn wait_for_analysis(session: &mut LspSession, timeout: Duration) -> Result<()> {
    let mut started = false;
    loop {
        let status = session.wait_notification(|_| true, timeout).await?;
        match status["params"]["isAnalyzing"].as_bool() {
            Some(true) => started = true,
            Some(false) if started => {
                session.poll().await?;
                if session.notifications().is_empty() {
                    return Ok(());
                }
            }
            Some(false) => {}
            None => bail!("The Dart server sent an analyzer status without isAnalyzing: {status}"),
        }
    }
}

#[cfg(test)]
mod tests;
