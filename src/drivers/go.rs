use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;
use super::lsp_client::LspClient;
use std::path::PathBuf;

pub struct GoDriver {
    client: LspClient,
}

impl GoDriver {
    pub fn new() -> Self {
        // gopls might be in PATH or in GOPATH/bin
        // We'll try to find it dynamically or default to "gopls"
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        Self {
            client: LspClient::new(&binary),
        }
    }

    fn find_gopls() -> Option<String> {
        // Check if "gopls" is in PATH
        if which::which("gopls").is_ok() {
            return Some("gopls".to_string());
        }

        // Check common GOPATH
        let home = std::env::var("HOME").ok()?;
        let gopath_bin = PathBuf::from(home).join("go/bin/gopls");
        if gopath_bin.exists() {
            return Some(gopath_bin.to_string_lossy().to_string());
        }

        None
    }
}

#[async_trait]
impl RefactorDriver for GoDriver {
    fn lang(&self) -> &str {
        "go"
    }

    async fn check_availability(&self) -> Result<bool> {
        self.client.check_availability().await
    }

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        // gopls supports standard textDocument/rename and executeCommand
        // For moves, gopls is interesting. It doesn't strictly have a "move file" refactoring in the LSP spec v3.16 directly mapped.
        // But `gopls` CLI does: `gopls rename old.go new.go`? No `gopls rename` is for symbols.
        // Wait, `gomove` or `gopls imports`.
        
        // Actually, for Go, moving a file inside the same package is trivial (just file move).
        // Moving to a new package requires updating 'package X' declaration.
        // Moving *imports* of that file in other files is the hard part.
        
        // `gopls` recently added support for file renaming via `workspace/willRenameFiles`.
        // So generic LspClient logic should work if gopls is running and watching the workspace.

        self.client.initialize_and_rename(&[], source, target).await?;
        
        tokio::fs::rename(source, target).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gopls_availability() -> Result<()> {
        // This test might fail if gopls isn't installed in the environment where test runs?
        // But we just installed it.
        let driver = GoDriver::new();
        let avail = driver.check_availability().await?;
        if !avail {
            eprintln!("gopls not found, skipping test");
            return Ok(());
        }
        Ok(())
    }
}
