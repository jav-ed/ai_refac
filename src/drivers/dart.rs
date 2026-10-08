use super::RefactorDriver;
use super::complete_filesystem_moves;
use crate::drivers::lsp::client::{
    LspClient, PendingChange, apply_pending_changes, collect_workspace_documents,
};
use anyhow::{Ok, Result, bail};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod directives;

#[derive(Default)]
pub struct DartDriver;

impl DartDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RefactorDriver for DartDriver {
    fn lang(&self) -> &str {
        "dart"
    }

    async fn check_availability(&self) -> Result<bool> {
        Ok(crate::servers::executable("dart", &std::env::current_dir()?).is_ok())
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let root = match root_path {
            Some(root) => std::path::absolute(root)?,
            None => std::env::current_dir()?,
        };

        // The command to start LSP is `dart language-server`
        let dart = crate::servers::executable("dart", &root)?;
        let client = LspClient::new(&dart.to_string_lossy());
        let changes = client
            .plan_file_renames(
                &["language-server"],
                file_map.clone(),
                Some(root.as_path()),
                "dart",
                &["dart"],
            )
            .await?;

        // Nothing has been written yet: a plan that leaves imports pointing at
        // files that will not exist is refused here, not discovered later.
        let dangling = dangling_imports(&root, &file_map, &changes)?;
        if !dangling.is_empty() {
            bail!(
                "The Dart server's plan would leave {} import(s) pointing at files that do not exist after the move. Nothing was changed. The server answers with a partial plan when it has not finished analysing (run the move again), and rewrites `package:` imports only when `.dart_tool/package_config.json` exists (run `dart pub get`).\n  {}",
                dangling.len(),
                dangling.join("\n  ")
            );
        }
        apply_pending_changes(changes).await?;

        complete_filesystem_moves(&file_map, Some(root.as_path())).await?;

        Ok(())
    }
}

/// The imports the plan would break, for the whole project as it will be.
fn dangling_imports(
    root: &Path,
    file_map: &[(String, String)],
    changes: &[PendingChange],
) -> Result<Vec<String>> {
    let absolute = |path: &str| root.join(path);
    let moves: Vec<(PathBuf, PathBuf)> = file_map
        .iter()
        .map(|(from, to)| (absolute(from), absolute(to)))
        .collect();
    let mut edits = HashMap::new();
    for change in changes {
        match change {
            PendingChange::TextEdit {
                path,
                edits: file_edits,
            } => {
                edits
                    .entry(path.clone())
                    .or_insert_with(Vec::new)
                    .extend(file_edits.iter().cloned());
            }
            PendingChange::ResourceOp(operation) => {
                bail!(
                    "The Dart server asked for a file operation that refac does not expect: {operation:?}"
                )
            }
        }
    }
    let files = collect_workspace_documents(root, &["dart"])?;
    directives::dangling_after(root, &files, &moves, &edits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Environment probe only — run with `cargo test -- --ignored` to verify dart is installed.
    #[tokio::test]
    #[ignore]
    async fn test_dart_availability() -> Result<()> {
        let driver = DartDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "dart not found in PATH");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
    async fn test_dart_move_updates_imports() -> Result<()> {
        let driver = DartDriver::new();
        assert!(driver.check_availability().await?, "dart not found");

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-dart-test-")
            .tempdir_in(std::env::temp_dir())?;
        fs::create_dir_all(temp_dir.path().join("lib/models"))?;
        fs::create_dir_all(temp_dir.path().join("lib/services"))?;
        fs::create_dir_all(temp_dir.path().join("lib/ui/screens"))?;
        fs::write(
            temp_dir.path().join("pubspec.yaml"),
            "name: demo\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n",
        )?;
        fs::write(
            temp_dir.path().join("lib/models/app_model.dart"),
            "class AppModel {}\n",
        )?;
        fs::write(
            temp_dir.path().join("lib/services/api_service.dart"),
            "import '../models/app_model.dart';\n\nclass ApiService {\n  AppModel load() => AppModel();\n}\n",
        )?;
        fs::write(
            temp_dir.path().join("lib/ui/screens/home_screen.dart"),
            "import '../../models/app_model.dart';\n\nclass HomeScreen {\n  AppModel value = AppModel();\n}\n",
        )?;

        driver
            .move_files(
                vec![(
                    "lib/models/app_model.dart".to_string(),
                    "lib/domain/app_model.dart".to_string(),
                )],
                Some(temp_dir.path()),
            )
            .await?;

        let api_service =
            fs::read_to_string(temp_dir.path().join("lib/services/api_service.dart"))?;
        let home_screen =
            fs::read_to_string(temp_dir.path().join("lib/ui/screens/home_screen.dart"))?;
        assert!(api_service.contains("../domain/app_model.dart"));
        assert!(home_screen.contains("../../domain/app_model.dart"));
        assert!(temp_dir.path().join("lib/domain/app_model.dart").exists());
        assert!(!temp_dir.path().join("lib/models/app_model.dart").exists());

        Ok(())
    }
}
