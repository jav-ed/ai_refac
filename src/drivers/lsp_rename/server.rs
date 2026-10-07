//! What the rename engine needs from a running language server. Kotlin's
//! server and the servers of the other languages differ in how they start and
//! when they are ready; once running, the engine talks to all of them alike.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

#[async_trait]
pub trait RenameServer: Send {
    /// A request that fails loudly instead of waiting forever. An error answer
    /// stays a downcastable `RpcError`.
    async fn request(&mut self, method: &str, params: Value) -> Result<Value>;

    /// Show the server a document's text without touching the disk.
    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()>;

    /// Return once the server has taken in the documents shown to it since
    /// the last call, so that its next answer is about their new text. A
    /// server that answers from the latest text at once needs nothing here.
    async fn settle(&mut self) -> Result<()> {
        Ok(())
    }

    /// Stop the server and delete whatever it created. Every rename ends here,
    /// so no server stays resident between two commands.
    async fn shutdown(self: Box<Self>);
}
