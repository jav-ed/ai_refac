//! Kotlin and Android backend. The JetBrains Kotlin language server does the
//! semantic work (moves rewrite package lines and imports, renames follow
//! references); refac wraps it with ordering, verification, rollback and the
//! Android parts the server does not touch.

use super::RefactorDriver;
use anyhow::Result;
use async_trait::async_trait;

pub mod android;
pub mod checks;
pub mod declarations;
pub mod moved;
pub mod moves;
pub mod plan;
pub mod project;
pub mod rename;
pub mod renames;
pub mod resync;
pub mod server;
pub mod stale;
pub mod survey;

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
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        self.move_files_with_notes(file_map, root_path).await?;
        Ok(())
    }

    /// The notes carry what only the caller can act on: names the server did
    /// not rewrite (build scripts, string literals) and steps it had to split.
    async fn move_files_with_notes(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<Vec<String>> {
        Ok(moves::move_files(&file_map, root_path).await?.notes)
    }
}
