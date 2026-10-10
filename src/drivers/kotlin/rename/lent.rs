//! A Kotlin server that someone else keeps (see `server::lend`), as the
//! rename engine sees a server. The engine stops its server at the end of a
//! rename; a lent one is not its to stop.

use crate::drivers::kotlin::server::SharedServer;
use crate::drivers::lsp::rename::server::RenameServer;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The engine holds the lock only for one call at a time.
pub struct Lent(pub SharedServer);

#[async_trait]
impl RenameServer for Lent {
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.0.lock().await.request(method, params).await
    }

    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        self.0.lock().await.sync_document(path, text).await
    }

    async fn after_apply(
        &mut self,
        edited: &[PathBuf],
        moves: &[(PathBuf, PathBuf)],
    ) -> Result<()> {
        RenameServer::after_apply(&mut *self.0.lock().await, edited, moves).await
    }

    /// The keeper stops the server, not the engine.
    async fn shutdown(self: Box<Self>) {}
}
