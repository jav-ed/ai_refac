//! Locating, starting and waiting for the JetBrains Kotlin language server.
//!
//! The server imports the Gradle build before it can answer. It signals that
//! with `intellij/workspaceImportState` (phase FINISHED) and then
//! `intellij/ready-for-test`; it keeps opening short "Indexing" progress
//! tokens forever, so those cannot tell readiness. Requests sent earlier are
//! answered `null`, which is why this module waits for the real signals and
//! never sleeps.

use crate::drivers::lsp_rename::server::RenameServer;
use crate::drivers::lsp_session::{LspSession, SessionConfig};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

pub const SERVER_ENV: &str = "REFAC_KOTLIN_SERVER";
pub const TIMEOUT_ENV: &str = "REFAC_KOTLIN_TIMEOUT_SECS";
const DEFAULT_TIMEOUT_SECS: u64 = 600;

pub struct Install {
    dir: PathBuf,
    pub build: String,
}

impl Install {
    fn executable(&self) -> PathBuf {
        self.dir.join("bin").join("intellij-server")
    }
}

const KEPT_NOTIFICATIONS: &[&str] = &[
    "intellij/workspaceImportState",
    "intellij/ready-for-test",
    "intellij/importLog",
];

/// The installed server, found like every language server (`crate::servers`):
/// the directory named by `REFAC_KOTLIN_SERVER`, which must hold
/// `bin/intellij-server` and `build.txt`. A missing install is reported with
/// the places looked at and the command that teaches the install.
pub fn locate() -> Result<Install> {
    locate_in(std::env::var_os(SERVER_ENV))
}

fn locate_in(configured: Option<OsString>) -> Result<Install> {
    let server = crate::servers::for_language("kotlin")?;
    let project = std::env::current_dir().context("Cannot read the current directory")?;
    let found = crate::servers::locate_with(server, &project, configured)?;
    let Some(build) = found.version.strip_prefix("ILS-") else {
        bail!(
            "{} does not name a Kotlin language server build (expected ILS-<number>, got `{}`). Run `refac doctor kotlin` to see which download refac expects.",
            found
                .folder
                .as_deref()
                .unwrap_or(&found.executable)
                .join("build.txt")
                .display(),
            found.version
        );
    };
    Ok(Install {
        dir: found
            .folder
            .context("The Kotlin server was found without its folder")?,
        build: build.to_string(),
    })
}

fn timeout() -> Result<Duration> {
    parse_timeout(std::env::var(TIMEOUT_ENV).ok().as_deref())
}

fn parse_timeout(configured: Option<&str>) -> Result<Duration> {
    let Some(value) = configured else {
        return Ok(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    };
    let seconds: u64 = value
        .parse()
        .ok()
        .filter(|seconds| *seconds > 0)
        .with_context(|| {
            format!("{TIMEOUT_ENV} must be a positive number of seconds, got `{value}`")
        })?;
    Ok(Duration::from_secs(seconds))
}

/// A running server with its throwaway system directory (about 250 MB of
/// caches that a second start does not reuse faster, so it is deleted).
pub struct KotlinServer {
    session: LspSession,
    pub build: String,
    timeout: Duration,
    system_dir: TempDir,
}

impl KotlinServer {
    /// Start the server on a Gradle root and return once the import is done.
    pub async fn start(install: &Install, project: &Path) -> Result<Self> {
        let timeout = timeout()?;
        let system_dir = tempfile::Builder::new().prefix("refac-kotlin-").tempdir()?;
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

    /// Stop the server and attach what the Gradle import printed.
    async fn fail(self, error: anyhow::Error) -> anyhow::Error {
        let import_log: Vec<String> = self
            .session
            .notifications()
            .iter()
            .filter(|message| message["method"] == "intellij/importLog")
            .filter_map(|message| message["params"]["message"].as_str())
            .map(|line| line.trim_end().to_string())
            // Stack frames bury the message that names the broken file.
            .filter(|line| !line.is_empty() && !line.trim_start().starts_with("at "))
            .collect();
        let shown = import_log[import_log.len().saturating_sub(25)..].join("\n");
        let build = self.build.clone();
        self.shutdown().await;
        error.context(format!(
            "Kotlin language server (build {build}) was not ready.\nLast Gradle import output:\n{shown}"
        ))
    }

    pub fn pid(&self) -> Option<u32> {
        self.session.pid()
    }

    /// A request that fails loudly instead of waiting forever. An error
    /// answer stays a downcastable `RpcError`.
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        tokio::time::timeout(self.timeout, self.session.request(method, params))
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "The Kotlin language server did not answer {method} within {} seconds",
                    self.timeout.as_secs()
                )
            })?
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.session.notify(method, params).await
    }

    pub async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        self.session.sync_document(path, text).await
    }

    pub async fn close_under(&mut self, path: &Path) -> Result<()> {
        self.session.close_under(path).await
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
