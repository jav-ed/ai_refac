use std::path::Path;

use anyhow::{Result, bail};
use async_trait::async_trait;

use super::RefactorDriver;

mod parser;
mod rewrite;

pub struct MarkdownDriver;

impl MarkdownDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RefactorDriver for MarkdownDriver {
    fn lang(&self) -> &str {
        "markdown"
    }

    async fn check_availability(&self) -> Result<bool> {
        Ok(true)
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
    ) -> Result<()> {
        for (source, _) in &file_map {
            let source_path = if let Some(root) = root_path {
                let candidate = Path::new(source);
                if candidate.is_absolute() {
                    candidate.to_path_buf()
                } else {
                    root.join(candidate)
                }
            } else {
                Path::new(source).to_path_buf()
            };

            if source_path.is_dir() {
                bail!("Markdown driver only supports file moves, not directories");
            }
        }

        rewrite::move_markdown_files(file_map, root_path).await
    }
}
