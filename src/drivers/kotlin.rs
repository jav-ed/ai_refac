//! Kotlin and Android backend. The JetBrains Kotlin language server does the
//! semantic work (moves rewrite package lines and imports, renames follow
//! references); refac wraps it with ordering, verification, rollback and the
//! Android parts the server does not touch.

use super::preview::copy::{CopyPlan, preview_on_copy};
use super::{MovePreview, RefactorDriver};
use anyhow::Result;
use async_trait::async_trait;

pub mod android;
pub mod checks;
pub mod declarations;
pub mod moves;
pub mod plan;
pub mod project;
pub mod rename;
pub mod resync;
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

    /// A Kotlin move runs in groups, each asked of the server after the one
    /// before was written, and then the Android layer reads the moved files. It
    /// is previewed by the real move on a copy of the project. The server
    /// imports the copy with Gradle, which takes as long as for a real move.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        preview_on_copy(
            root_path,
            &file_map,
            CopyPlan {
                tool_state: &[".gradle", ".kotlin"],
                scratch: &["build"],
            },
            |pairs, copy| async move {
                Ok(moves::move_files(&pairs, Some(copy.as_path())).await?.notes)
            },
        )
        .await
    }
}
