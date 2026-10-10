//! Locating, starting and waiting for the JetBrains Kotlin language server.
//!
//! The server imports the Gradle build before it can answer. It signals that
//! with `intellij/workspaceImportState` (phase FINISHED) and then
//! `intellij/ready-for-test`; it keeps opening short "Indexing" progress
//! tokens forever, so those cannot tell readiness. Requests sent earlier are
//! answered `null`, which is why this module waits for the real signals and
//! never sleeps.

mod cache;
mod capabilities;
mod install;
mod lend;
mod mirror;

use crate::drivers::lsp::rename::server::RenameServer;
use crate::drivers::lsp::session::{LspSession, SessionConfig};
use anyhow::Result;
use async_trait::async_trait;
use cache::SystemDir;
use capabilities::{check_capabilities, client_capabilities};
use install::{gradle_idle_ms, timeout};
use mirror::Mirror;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What the server's Gradle import prints for a build it cannot model.
const UNREADABLE_TARGETS: &str = "Failed to find 'target' in Kotlin extension";

/// What to tell the user about a change on this Gradle root: that it was
/// planned on a plain-JVM copy, when the build is multiplatform.
pub fn multiplatform_note(root: &Path) -> Result<Option<String>> {
    Ok(Mirror::applies_to(root)?.then(Mirror::note))
}

pub use cache::CACHE_ENV;
pub use install::{GRADLE_IDLE_ENV, Install, SERVER_ENV, TIMEOUT_ENV, locate};
pub use lend::{SharedServer, lend, lent_for, recall};
pub use mirror::{refuse_expect_actual, refuse_expect_actual_symbol};

const KEPT_NOTIFICATIONS: &[&str] = &[
    "intellij/workspaceImportState",
    "intellij/ready-for-test",
    "intellij/importLog",
];

/// A running server with the system directory it works in (its caches and
/// indexes, about 120 MB; a run that ends cleanly keeps them for the next, see
/// `cache`).
pub struct KotlinServer {
    session: LspSession,
    pub build: String,
    timeout: Duration,
    system_dir: SystemDir,
    /// Set for a Kotlin Multiplatform build, which the server cannot import:
    /// it then works on a plain-JVM copy and everything crossing this boundary
    /// is translated (see `mirror`).
    mirror: Option<Mirror>,
    /// Every file the server was shown a text of or told a change of. A
    /// caller that keeps the server and rolls the disk back after a failure
    /// tells it again what the disk has for each of them.
    told: BTreeSet<PathBuf>,
}

impl KotlinServer {
    /// Start the server on a Gradle root and return once the import is done.
    pub async fn start(install: &Install, project: &Path) -> Result<Self> {
        Self::start_with(install, project, gradle_idle_ms()?).await
    }

    /// The same, with the time, in milliseconds, that the Gradle daemon of the
    /// import may sit idle after the run (`start` reads it from
    /// `REFAC_KOTLIN_GRADLE_IDLE_SECS`). A caller that starts servers one run
    /// after the other, such as the test programs, passes minutes: the next
    /// import then finds the daemon running.
    pub async fn start_with(
        install: &Install,
        project: &Path,
        gradle_idle_ms: u64,
    ) -> Result<Self> {
        let timeout = timeout()?;
        let system_dir = SystemDir::open(&install.build)?;
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
            // Without `indexDir` the server keeps a full index of the JDK and
            // the libraries for each project path (120 MB each, and none of
            // it is found again under another path); with it, one shared index
            // that every project and every copy of a project starts from.
            initialization_options: Some(json!({ "indexDir": system_dir.path() })),
            env: vec![(
                "JAVA_TOOL_OPTIONS".to_string(),
                java_tool_options(
                    std::env::var("JAVA_TOOL_OPTIONS").ok().as_deref(),
                    gradle_idle_ms,
                ),
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
            told: BTreeSet::new(),
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
        self.abandon().await;
        error.context(format!(
            "Kotlin language server (build {build}) was not ready.\nLast Gradle import output:\n{shown}{about_mirror}"
        ))
    }

    pub fn pid(&self) -> Option<u32> {
        self.session.pid()
    }

    /// The files the server was shown or told a change of (see `told`).
    pub fn told(&self) -> Vec<PathBuf> {
        self.told.iter().cloned().collect()
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
        if method == "workspace/didChangeWatchedFiles" {
            self.told.extend(event_paths(&params)?);
        }
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
        self.told.insert(path.to_path_buf());
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

    /// Stop the server. One that left by itself has written its caches
    /// completely, and they are kept for the next run (see `cache`). A cache
    /// that could not be kept costs the next run its speed, not this one its
    /// result, so it is reported and not an error.
    pub async fn shutdown(self) {
        if self.session.shutdown().await
            && let Err(error) = self.system_dir.keep()
        {
            tracing::warn!("The Kotlin server's caches were not kept: {error:#}");
        }
    }

    /// Stop a server whose run failed; its caches are not kept.
    async fn abandon(self) {
        self.session.shutdown().await;
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

/// The paths a `workspace/didChangeWatchedFiles` notification names.
fn event_paths(params: &Value) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for change in params["changes"].as_array().into_iter().flatten() {
        let uri = change["uri"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("A file event without a uri: {change}"))?;
        let path = url::Url::parse(uri)?
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("A file event for something that is no path: {uri}"))?;
        paths.push(path);
    }
    Ok(paths)
}

/// The server imports the Gradle build through a Gradle daemon, and Gradle
/// keeps that daemon running for three hours after the import (about 0.5 GB of
/// memory). Nothing refac starts may outlive the command, so the server is
/// started with an idle timeout of a few seconds and the daemon stops itself
/// once the import is done; the semantic requests that follow are answered
/// by the server, not by Gradle. A caller who runs several Kotlin commands one
/// after the other can ask for longer (`REFAC_KOTLIN_GRADLE_IDLE_SECS`): the
/// next import then finds the daemon running and is about 7 seconds faster.
/// Any `JAVA_TOOL_OPTIONS` the user has stay.
fn java_tool_options(existing: Option<&str>, idle_ms: u64) -> String {
    let ours = format!("-Dorg.gradle.daemon.idletimeout={idle_ms}");
    match existing
        .map(str::trim)
        .filter(|options| !options.is_empty())
    {
        Some(options) => format!("{options} {ours}"),
        None => ours,
    }
}

fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("java") => "java",
        _ => "kotlin",
    }
}

#[cfg(test)]
mod tests;
