//! Proving a found server can start: the `initialize` conversation on an
//! empty folder, then a clean shutdown. It takes a second or two and catches
//! what a version command cannot: a server that runs but cannot serve.

use super::locate::Located;
use super::{Find, Server};
use crate::drivers::lsp::session::{LspSession, SessionConfig};
use anyhow::{Result, bail};
use std::path::Path;
use std::time::{Duration, Instant};

const START_TIMEOUT: Duration = Duration::from_secs(60);

pub enum StartCheck {
    /// Why the check was not made.
    Skipped(String),
    Started {
        seconds: f32,
    },
    Failed(String),
}

pub async fn start_check(server: &'static Server, located: &Located, project: &Path) -> StartCheck {
    let Find::Executable { launch, .. } = &server.find else {
        return StartCheck::Skipped(match server.find {
            Find::Folder { .. } => "starting it imports a Gradle project and takes about 30 seconds; run a rename to see it work".to_string(),
            _ => "it is started by refac's own helper scripts".to_string(),
        });
    };
    let began = Instant::now();
    match tokio::time::timeout(START_TIMEOUT, handshake(server, launch, located, project)).await {
        Ok(Ok(())) => StartCheck::Started {
            seconds: began.elapsed().as_secs_f32(),
        },
        Ok(Err(error)) => StartCheck::Failed(format!("{error:#}")),
        Err(_) => StartCheck::Failed(format!(
            "it did not answer within {} seconds",
            START_TIMEOUT.as_secs()
        )),
    }
}

async fn handshake(
    server: &'static Server,
    launch: &super::Launch,
    located: &Located,
    project: &Path,
) -> Result<()> {
    let empty = tempfile::Builder::new().prefix("refac-doctor-").tempdir()?;
    let (session, init) = LspSession::start(SessionConfig {
        name: server.name,
        executable: &located.executable,
        args: launch.args.iter().map(|arg| arg.to_string()).collect(),
        // Rustup picks the toolchain from the directory the server starts in.
        cwd: project,
        root: empty.path(),
        capabilities: launch.profile.capabilities(false),
        keep_notifications: launch.profile.kept_notifications(),
        language_id: launch.profile.language_id(),
        env: Vec::new(),
    })
    .await?;
    let missing = launch.profile.missing_capabilities(&init["capabilities"]);
    session.shutdown().await;
    if !missing.is_empty() {
        bail!(
            "it started but does not offer {}; it is too old or the wrong program",
            missing.join(", ")
        );
    }
    Ok(())
}
