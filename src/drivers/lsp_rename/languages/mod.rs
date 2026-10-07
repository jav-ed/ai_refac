//! The languages the rename engine serves with a server found by
//! `crate::servers`. Kotlin has its own home (`drivers/kotlin/rename.rs`)
//! because of the Android work it adds.

mod dart;
mod go;
mod python;
mod rust;

use super::project_server::{Launch, ProjectServer};
use super::server::RenameServer;
use crate::servers::{self, Find};
use anyhow::{Result, bail};
use std::path::Path;

pub use dart::Dart;
pub use go::Go;
pub use python::Python;
pub use rust::Rust;

/// Find the server of `language` (a name `refac doctor` knows) and start it
/// on the project with `file` open. A server that cannot be found is
/// reported with where it was looked for and the command that teaches the
/// install.
pub(super) async fn start_server(
    language: &str,
    root: &Path,
    file: &Path,
) -> Result<Box<dyn RenameServer>> {
    let server = servers::for_language(language)?;
    let located = servers::locate(server, root)?;
    let Find::Executable { launch, .. } = &server.find else {
        bail!("{} is not started as a plain executable", server.name);
    };
    let documents = [file.to_path_buf()];
    let started = ProjectServer::start(Launch {
        server: launch.profile,
        executable: &located.executable,
        args: launch.args.iter().map(|arg| arg.to_string()).collect(),
        cwd: root,
        root,
        documents: &documents,
    })
    .await?;
    Ok(Box::new(started))
}

#[cfg(test)]
mod tests;
