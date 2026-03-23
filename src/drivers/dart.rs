use super::RefactorDriver;
use super::complete_filesystem_moves;
use super::lsp_client::LspClient;
use anyhow::{Ok, Result};
use async_trait::async_trait;

pub struct DartDriver {
    client: LspClient,
}

impl DartDriver {
    pub fn new() -> Self {
        Self {
            client: LspClient::new("dart"),
        }
    }
}

#[async_trait]
impl RefactorDriver for DartDriver {
    fn lang(&self) -> &str {
        "dart"
    }

    async fn check_availability(&self) -> Result<bool> {
        match tokio::process::Command::new("dart")
            .arg("--version")
            .output()
            .await
        {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        // The command to start LSP is `dart language-server`
        self.client
            .initialize_and_rename_files(
                &["language-server"],
                file_map.clone(),
                root_path,
                Some("dart"),
                &["dart"],
            )
            .await?;

        complete_filesystem_moves(&file_map, root_path).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_dart_availability() -> Result<()> {
        let driver = DartDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "dart should be available");
        Ok(())
    }

    #[tokio::test]
    async fn test_dart_move_updates_imports() -> Result<()> {
        let driver = DartDriver::new();
        if !driver.check_availability().await? {
            eprintln!("dart not found, skipping test");
            return Ok(());
        }

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
