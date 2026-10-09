//! Locating, starting and waiting for the JetBrains Kotlin language server.
//!
//! The server imports the Gradle build before it can answer. It signals that
//! with `intellij/workspaceImportState` (phase FINISHED) and then
//! `intellij/ready-for-test`; it keeps opening short "Indexing" progress
//! tokens forever, so those cannot tell readiness. Requests sent earlier are
//! answered `null`, which is why this module waits for the real signals and
//! never sleeps.

mod install;
mod mirror;

use crate::drivers::lsp::rename::server::RenameServer;
use crate::drivers::lsp::session::{LspSession, SessionConfig};
use anyhow::{Result, bail};
use async_trait::async_trait;
use install::timeout;
use mirror::Mirror;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

/// What the server's Gradle import prints for a build it cannot model.
const UNREADABLE_TARGETS: &str = "Failed to find 'target' in Kotlin extension";

/// What to tell the user about a change on this Gradle root: that it was
/// planned on a plain-JVM copy, when the build is multiplatform.
pub fn multiplatform_note(root: &Path) -> Result<Option<String>> {
    Ok(Mirror::applies_to(root)?.then(Mirror::note))
}

pub use install::{Install, SERVER_ENV, TIMEOUT_ENV, locate};
pub use mirror::{refuse_expect_actual, refuse_expect_actual_symbol};

const KEPT_NOTIFICATIONS: &[&str] = &[
    "intellij/workspaceImportState",
    "intellij/ready-for-test",
    "intellij/importLog",
];

/// A running server with its throwaway system directory (about 250 MB of
/// caches that a second start does not reuse faster, so it is deleted).
pub struct KotlinServer {
    session: LspSession,
    pub build: String,
    timeout: Duration,
    system_dir: TempDir,
    /// Set for a Kotlin Multiplatform build, which the server cannot import:
    /// it then works on a plain-JVM copy and everything crossing this boundary
    /// is translated (see `mirror`).
    mirror: Option<Mirror>,
}

impl KotlinServer {
    /// Start the server on a Gradle root and return once the import is done.
    pub async fn start(install: &Install, project: &Path) -> Result<Self> {
        let timeout = timeout()?;
        let system_dir = tempfile::Builder::new().prefix("refac-kotlin-").tempdir()?;
        let mirror = Mirror::for_project(project)?;
        let project = mirror.as_ref().map_or(project, Mirror::root);
        let (mut session, init) = LspSession::start(SessionConfig {
            name: "Kotlin language server",
            executable: &install.executable(),
            args: vec![
                "--stdio".to_string(),
                format!("--system-path={}", system_dir.path().display()),
            ],
            cwd: project,
            root: project,
            capabilities: client_capabilities(),
            keep_notifications: KEPT_NOTIFICATIONS,
            language_id,
            env: vec![(
                "JAVA_TOOL_OPTIONS".to_string(),
                java_tool_options(std::env::var("JAVA_TOOL_OPTIONS").ok().as_deref()),
            )],
        })
        .await?;
        check_capabilities(&init["capabilities"])?;
        session.initialized().await?;
        let build = install.build.clone();
        let server = Self {
            session,
            build,
            timeout,
            system_dir,
            mirror,
        };
        server.wait_until_ready().await
    }

    async fn wait_until_ready(mut self) -> Result<Self> {
        let import = self
            .session
            .wait_notification(
                |message| {
                    message["method"] == "intellij/workspaceImportState"
                        && message["params"]["phase"] == "FINISHED"
                },
                self.timeout,
            )
            .await;
        let import = match import {
            Ok(import) => import,
            Err(error) => return Err(self.fail(error).await),
        };
        // The import's own output explains what the server could not model;
        // `RUST_LOG=debug` shows it.
        let log = self.import_log();
        for line in &log {
            tracing::debug!("Gradle import: {line}");
        }
        // The import still ends "successfully" when it cannot read the Kotlin
        // targets, and the server then knows no source. The mirror avoids
        // this for the multiplatform layouts refac recognises.
        if self.mirror.is_none() && log.iter().any(|line| line.contains(UNREADABLE_TARGETS)) {
            let error = anyhow::anyhow!(
                "The Kotlin server could not read the Kotlin targets of this Gradle build (`{UNREADABLE_TARGETS}`). That is how it reports a Kotlin Multiplatform build, and it would then refuse every move. refac recognises a multiplatform build by a source set named commonMain, commonTest or <target>Main; this project has none"
            );
            return Err(self.fail(error).await);
        }
        if let Some(folder) = import["params"]["folders"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|folder| folder["status"] != "SUCCESS")
        {
            let error = anyhow::anyhow!(
                "The Gradle import failed ({}) for {}",
                folder["status"],
                folder["folderUri"]
            );
            return Err(self.fail(error).await);
        }
        let ready = self
            .session
            .wait_notification(
                |message| message["method"] == "intellij/ready-for-test",
                self.timeout,
            )
            .await;
        match ready {
            Ok(_) => Ok(self),
            Err(error) => Err(self.fail(error).await),
        }
    }

    /// What the Gradle import printed, without blank lines and stack frames
    /// (those bury the message that names the broken file).
    fn import_log(&self) -> Vec<String> {
        self.session
            .notifications()
            .iter()
            .filter(|message| message["method"] == "intellij/importLog")
            .filter_map(|message| message["params"]["message"].as_str())
            .map(|line| line.trim_end().to_string())
            .filter(|line| !line.is_empty() && !line.trim_start().starts_with("at "))
            .collect()
    }

    /// Stop the server and attach what the Gradle import printed.
    async fn fail(self, error: anyhow::Error) -> anyhow::Error {
        let import_log = self.import_log();
        let shown = import_log[import_log.len().saturating_sub(25)..].join("\n");
        let build = self.build.clone();
        let about_mirror = if self.mirror.is_some() {
            "\nThis is a Kotlin Multiplatform build: the server imported refac's plain-JVM copy of it, so the output above is about the copy."
        } else {
            ""
        };
        self.shutdown().await;
        error.context(format!(
            "Kotlin language server (build {build}) was not ready.\nLast Gradle import output:\n{shown}{about_mirror}"
        ))
    }

    pub fn pid(&self) -> Option<u32> {
        self.session.pid()
    }

    /// A request that fails loudly instead of waiting forever. An error
    /// answer stays a downcastable `RpcError`. Callers speak in the real
    /// project's paths; a mirror translates them for the server and back.
    pub async fn request(&mut self, method: &str, mut params: Value) -> Result<Value> {
        if let Some(mirror) = &self.mirror {
            mirror.uris_to_mirror(&mut params);
        }
        let mut answer =
            tokio::time::timeout(self.timeout, self.session.request(method, params.clone()))
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "The Kotlin language server did not answer {method} within {} seconds",
                        self.timeout.as_secs()
                    )
                })??;
        if let Some(mirror) = &self.mirror {
            if !answer.is_null() {
                mirror::keep_imports(method, &mut answer, &params, &|path| mirror.to_real(path))?;
            }
            mirror.uris_to_real(&mut answer);
        }
        Ok(answer)
    }

    pub async fn notify(&mut self, method: &str, mut params: Value) -> Result<()> {
        if let Some(mirror) = &self.mirror {
            params = if method == "workspace/didChangeWatchedFiles" {
                mirror.follow_changes(&params)?
            } else {
                mirror.uris_to_mirror(&mut params);
                params
            };
        }
        self.session.notify(method, params).await
    }

    pub async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        let Some(mirror) = &self.mirror else {
            return self.session.sync_document(path, text).await;
        };
        let seen = mirror.write_document(path, text)?;
        self.session
            .sync_document(&mirror.to_mirror(path), &seen)
            .await
    }

    pub async fn close_under(&mut self, path: &Path) -> Result<()> {
        let path = self
            .mirror
            .as_ref()
            .map_or_else(|| path.to_path_buf(), |mirror| mirror.to_mirror(path));
        self.session.close_under(&path).await
    }

    pub async fn shutdown(self) {
        self.session.shutdown().await;
        // Best effort: the server may still hold a file for a moment.
        let _ = self.system_dir.close();
    }
}

/// The rename engine drives the Kotlin server through the same calls as the
/// other languages' servers.
#[async_trait]
impl RenameServer for KotlinServer {
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        KotlinServer::request(self, method, params).await
    }

    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        KotlinServer::sync_document(self, path, text).await
    }

    async fn after_apply(
        &mut self,
        edited: &[PathBuf],
        moves: &[(PathBuf, PathBuf)],
    ) -> Result<()> {
        super::resync::after_rename(self, edited, moves).await
    }

    async fn shutdown(self: Box<Self>) {
        KotlinServer::shutdown(*self).await;
    }
}

/// How long, in milliseconds, a Gradle daemon that the server's build import
/// started may sit idle before it stops itself.
const GRADLE_DAEMON_IDLE_MS: u64 = 10_000;

/// The server imports the Gradle build through a Gradle daemon, and Gradle
/// keeps that daemon running for three hours after the import (about 0.5 GB of
/// memory). Nothing refac starts may outlive the command, so the server is
/// started with an idle timeout of a few seconds and the daemon stops itself
/// once the import is done; the semantic requests that follow are answered
/// by the server, not by Gradle. Any `JAVA_TOOL_OPTIONS` the user has stay.
fn java_tool_options(existing: Option<&str>) -> String {
    let ours = format!("-Dorg.gradle.daemon.idletimeout={GRADLE_DAEMON_IDLE_MS}");
    match existing
        .map(str::trim)
        .filter(|options| !options.is_empty())
    {
        Some(options) => format!("{options} {ours}"),
        None => ours,
    }
}

fn client_capabilities() -> Value {
    json!({
        "workspace": {
            "applyEdit": true,
            "workspaceEdit": {
                "documentChanges": true,
                "resourceOperations": ["create", "rename", "delete"],
            },
            "workspaceFolders": true,
            "configuration": true,
            "fileOperations": { "willRename": true },
        },
        "window": { "workDoneProgress": true },
        "textDocument": {
            "rename": { "prepareSupport": true },
            "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
        },
    })
}

fn check_capabilities(capabilities: &Value) -> Result<()> {
    if capabilities["workspace"]["fileOperations"]["willRename"].is_null() {
        bail!("The Kotlin language server does not offer workspace/willRenameFiles");
    }
    if capabilities["renameProvider"]["prepareProvider"] != true {
        bail!("The Kotlin language server does not offer prepareRename");
    }
    if capabilities["referencesProvider"].is_null()
        || capabilities["documentSymbolProvider"].is_null()
    {
        bail!("The Kotlin language server does not offer references and document symbols");
    }
    Ok(())
}

fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("java") => "java",
        _ => "kotlin",
    }
}

#[cfg(test)]
mod tests;
