//! Kotlin and Android backend. The JetBrains Kotlin language server does the
//! semantic work (moves rewrite package lines and imports, renames follow
//! references); refac wraps it with ordering, verification, rollback and the
//! Android parts the server does not touch.

use super::RefactorDriver;
use anyhow::{Result, bail};
use async_trait::async_trait;

pub mod project;
pub mod server;

pub struct KotlinDriver;

#[async_trait]
impl RefactorDriver for KotlinDriver {
    fn lang(&self) -> &str {
        "kotlin"
    }

    /// The server is the one external dependency and has no auto-download, so
    /// a missing install is an error that carries the install steps.
    async fn check_availability(&self) -> Result<bool> {
        server::locate()?;
        Ok(true)
    }

    async fn move_files(
        &self,
        _file_map: Vec<(String, String)>,
        _root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        bail!("Kotlin moves are not implemented yet")
    }
}
