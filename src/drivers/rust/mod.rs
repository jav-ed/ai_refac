use super::{RefactorDriver, complete_filesystem_moves};
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::Path;

mod apply;
mod availability;
mod layout;
mod module_graph;
mod planner;
mod references;
mod rename;
mod validation;
mod workspace;

pub use planner::move_module;

use availability::rust_analyzer_command;
use rename::{RustSymbolRenameRequest, build_symbol_rename_request};

pub struct RustDriver;

impl RustDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RefactorDriver for RustDriver {
    fn lang(&self) -> &str {
        "rust"
    }

    async fn check_availability(&self) -> Result<bool> {
        availability::rust_analyzer_is_available().await
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
    ) -> Result<()> {
        let root_dir = root_path
            .map(Path::to_path_buf)
            .unwrap_or(std::env::current_dir()?);
        let client = super::lsp_client::LspClient::new(&rust_analyzer_command());
        let mut lsp_batch: Vec<(String, String, RustSymbolRenameRequest)> = Vec::new();

        for (source, target) in &file_map {
            let source_abs = rename::absolute_path(&root_dir, Path::new(source));
            let target_abs = rename::absolute_path(&root_dir, Path::new(target));

            if source_abs.parent() != target_abs.parent() {
                bail!(
                    "Rust file move crosses a module boundary: {} -> {}. Use `refac move-module --project-path {} crate::<source> crate::<target>` so refac can move the complete logical module without introducing #[path] shims.",
                    source_abs.display(),
                    target_abs.display(),
                    root_dir.display()
                );
            }

            if let Some(request) = build_symbol_rename_request(&root_dir, &source_abs, &target_abs)?
            {
                lsp_batch.push((source.clone(), target.clone(), request));
                continue;
            }

            client
                .initialize_and_rename_files(
                    &[],
                    vec![(source.clone(), target.clone())],
                    Some(root_dir.as_path()),
                    Some("rust"),
                    &["rs"],
                )
                .await?;
            complete_filesystem_moves(
                &[(source.clone(), target.clone())],
                Some(root_dir.as_path()),
            )
            .await?;
        }

        if lsp_batch.is_empty() {
            return Ok(());
        }

        let requests = lsp_batch
            .iter()
            .map(|(source, target, request)| {
                let source_abs = rename::absolute_path(&root_dir, Path::new(source));
                let target_abs = rename::absolute_path(&root_dir, Path::new(target));
                let mut pending_moves = std::collections::HashMap::new();
                pending_moves.insert(target_abs, source_abs);

                super::lsp_client::SymbolRenameRequest {
                    document_path: request.document_path.clone(),
                    position: request.position,
                    new_name: request.new_name.clone(),
                    pending_moves,
                }
            })
            .collect();

        client
            .initialize_and_rename_symbols_batch(&[], Some(root_dir.as_path()), requests, "rust")
            .await?;

        let moves = lsp_batch
            .into_iter()
            .map(|(source, target, _)| (source, target))
            .collect::<Vec<_>>();
        complete_filesystem_moves(&moves, Some(root_dir.as_path())).await
    }
}
