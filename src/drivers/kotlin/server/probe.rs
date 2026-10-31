//! Proving the installed server starts, for `refac doctor kotlin`.
//!
//! Only `initialize` is sent, on an empty folder. The `initialized` message is
//! what makes the server import a Gradle project (20 to 45 seconds), and it is
//! never sent. A build whose license ran out leaves before it answers
//! ("This build of kotlin-server has expired"), so only starting it tells it
//! from a good one: `build.txt` names the build either way.

use super::capabilities::{check_capabilities, client_capabilities};
use super::expired::explain;
use super::{language_id, server_args};
use crate::drivers::lsp::session::{LspSession, SessionConfig};
use anyhow::{Context, Result};
use serde_json::json;
use std::path::Path;

/// Start the server at `executable`, check what it offers, and stop it. The
/// error of a server that does not start carries what it printed.
pub async fn check_start(executable: &Path) -> Result<()> {
    let scratch = tempfile::Builder::new()
        .prefix("refac-doctor-")
        .tempdir()
        .context("Cannot create a folder for the Kotlin start check")?;
    let project = scratch.path().join("project");
    let system = scratch.path().join("system");
    for dir in [&project, &system] {
        std::fs::create_dir(dir).with_context(|| format!("Cannot create {}", dir.display()))?;
    }
    let (session, init) = LspSession::start(SessionConfig {
        name: "Kotlin language server",
        executable,
        args: server_args(&system),
        cwd: &project,
        root: &project,
        capabilities: client_capabilities(),
        keep_notifications: &[],
        language_id,
        env: Vec::new(),
        initialization_options: Some(json!({ "indexDir": system })),
    })
    .await
    .map_err(|error| explain(error, executable))?;
    let offered = check_capabilities(&init["capabilities"]);
    session.shutdown().await;
    offered
}

#[cfg(test)]
mod tests;
