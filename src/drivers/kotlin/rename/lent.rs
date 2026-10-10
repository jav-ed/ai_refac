//! A Kotlin server that someone else started and keeps. The rename engine
//! starts a server per command and stops it at the end, which is right for a
//! command and costs half a minute of Gradle import. A caller that runs many
//! renames against one project (the tests of this repository) starts the
//! server once, hands it to the engine through this wrapper, and decides when
//! it stops.

use crate::drivers::kotlin::server::KotlinServer;
use crate::drivers::lsp::rename::server::RenameServer;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A running server, shared between its keeper and the engine. The engine
/// holds the lock only for one call at a time.
pub type SharedServer = Arc<Mutex<KotlinServer>>;

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
