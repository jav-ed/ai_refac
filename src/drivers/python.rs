use super::RefactorDriver;
use anyhow::Result;
use std::result::Result::Ok;
use async_trait::async_trait;
use super::python_rope::RopeDriver;
use super::python_pyrefly::PyreflyDriver;

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

    async fn move_files(&self, file_map: Vec<(String, String)>) -> Result<()> {
        // Try Pyrefly first
        if self.pyrefly.check_availability().await.unwrap_or(false) {
             match self.pyrefly.move_files(file_map.clone()).await {
                 Ok(_) => return Ok(()),
                 Err(e) => {
                     tracing::warn!("Pyrefly failed, falling back to Rope: {}", e);
                 }
             }
        }
        
        // Fallback to Rope
        self.rope.move_files(file_map).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_dispatcher_availability() -> Result<()> {
        let driver = PythonDriver::new();
        // Should be true if either is available
        assert!(driver.check_availability().await?);
        Ok(())
    }
}
