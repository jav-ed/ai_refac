use super::RefactorDriver;
use super::python_pyrefly::PyreflyDriver;
use super::python_rope::RopeDriver;
use anyhow::Result;
use async_trait::async_trait;
use std::result::Result::Ok;

pub struct PythonDriver {
    rope: RopeDriver,
    pyrefly: PyreflyDriver,
}

impl PythonDriver {
    pub fn new() -> Self {
        Self {
            rope: RopeDriver,
            pyrefly: PyreflyDriver::new(),
        }
    }
}

#[async_trait]
impl RefactorDriver for PythonDriver {
    fn lang(&self) -> &str {
        "python"
    }

    async fn check_availability(&self) -> Result<bool> {
        // Check if at least one is available
        let rope_avail = self.rope.check_availability().await.unwrap_or(false);
        let pyrefly_avail = self.pyrefly.check_availability().await.unwrap_or(false);
        Ok(rope_avail || pyrefly_avail)
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        // Prefer Rope because it currently updates imports more reliably for
        // file moves than the Pyrefly `willRenameFiles` path.
        if self.rope.check_availability().await.unwrap_or(false) {
            match self.rope.move_files(file_map.clone(), root_path).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!("Rope failed, falling back to Pyrefly: {}", e);
                }
            }
        }

        // Fallback to Pyrefly
        self.pyrefly.move_files(file_map, root_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_python_dispatcher_availability() -> Result<()> {
        let driver = PythonDriver::new();
        // Should be true if either is available
        assert!(driver.check_availability().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_python_move_updates_imports() -> Result<()> {
        let driver = PythonDriver::new();
        if !driver.check_availability().await? {
            eprintln!("No Python refactor backend found, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-python-test-")
            .tempdir_in(std::env::temp_dir())?;
        fs::create_dir_all(temp_dir.path().join("lib/db"))?;
        fs::create_dir_all(temp_dir.path().join("lib/services"))?;
        fs::write(temp_dir.path().join("lib/__init__.py"), "")?;
        fs::write(temp_dir.path().join("lib/db/__init__.py"), "")?;
        fs::write(temp_dir.path().join("lib/services/__init__.py"), "")?;
        fs::write(
            temp_dir.path().join("lib/db/database.py"),
            "class DatabaseConnection:\n    pass\n",
        )?;
        fs::write(
            temp_dir.path().join("lib/services/order_service.py"),
            "from lib.db.database import DatabaseConnection\n\nclass OrderService:\n    def __init__(self, db: DatabaseConnection):\n        self.db = db\n",
        )?;
        fs::write(
            temp_dir.path().join("main.py"),
            "from lib.db.database import DatabaseConnection\n\nDB = DatabaseConnection()\n",
        )?;

        driver
            .move_files(
                vec![(
                    "lib/db/database.py".to_string(),
                    "lib/services/database.py".to_string(),
                )],
                Some(temp_dir.path()),
            )
            .await?;

        let order_service =
            fs::read_to_string(temp_dir.path().join("lib/services/order_service.py"))?;
        let main = fs::read_to_string(temp_dir.path().join("main.py"))?;
        assert!(order_service.contains("from lib.services.database import DatabaseConnection"));
        assert!(main.contains("from lib.services.database import DatabaseConnection"));
        assert!(temp_dir.path().join("lib/services/database.py").exists());
        assert!(!temp_dir.path().join("lib/db/database.py").exists());

        let import_check = std::process::Command::new("python3")
            .arg("-c")
            .arg("import main")
            .current_dir(temp_dir.path())
            .output()?;
        assert!(
            import_check.status.success(),
            "python import failed: {}",
            String::from_utf8_lossy(&import_check.stderr)
        );

        Ok(())
    }
}
