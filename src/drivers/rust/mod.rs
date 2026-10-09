use super::{MovePreview, RefactorDriver, complete_filesystem_moves};
use crate::drivers::lsp::client::{SymbolRenameRequest, summarize};
use analysis::lockfile::NewLockfiles;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::Path;

mod analysis;
mod compile_check;
mod edits;
mod planner;
mod rename;
mod target_parent;
mod transaction;

pub use planner::{MoveMode, move_module};

use rename::{RustSymbolRenameRequest, build_symbol_rename_request};

#[derive(Default)]
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
        Ok(crate::servers::executable("rust", &std::env::current_dir()?).is_ok())
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
    ) -> Result<()> {
        let root_dir = root_path
            .map(Path::to_path_buf)
            .unwrap_or(std::env::current_dir()?);
        let binary = crate::servers::executable("rust", &root_dir)?;
        let client = crate::drivers::lsp::client::LspClient::new(&binary.to_string_lossy());
        let mut lsp_batch: Vec<(String, String, RustSymbolRenameRequest)> = Vec::new();

        for (source, target) in &file_map {
            let source_abs = rename::absolute_path(&root_dir, Path::new(source));
            let target_abs = rename::absolute_path(&root_dir, Path::new(target));
            refuse_module_boundary(&root_dir, &source_abs, &target_abs)?;

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
                    "rust",
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

        let requests = symbol_requests(&root_dir, &lsp_batch);
        client
            .initialize_and_rename_symbols_batch(&[], Some(root_dir.as_path()), requests, "rust")
            .await?;

        let moves = lsp_batch
            .into_iter()
            .map(|(source, target, _)| (source, target))
            .collect::<Vec<_>>();
        complete_filesystem_moves(&moves, Some(root_dir.as_path())).await
    }

    /// The same two ways the real move has: a rename of the module's file is a
    /// symbol rename, anything else is the server's answer to a file rename.
    /// Every request is planned against the files as they are now.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
    ) -> Result<MovePreview> {
        let root_dir = root_path
            .map(Path::to_path_buf)
            .unwrap_or(std::env::current_dir()?);
        // Cargo writes a missing Cargo.lock while rust-analyzer loads the
        // workspace; a dry run changes no file, so it removes the one it caused.
        let _lockfiles = NewLockfiles::watch(&root_dir);
        let binary = crate::servers::executable("rust", &root_dir)?;
        let client = crate::drivers::lsp::client::LspClient::new(&binary.to_string_lossy());
        let mut lsp_batch: Vec<(String, String, RustSymbolRenameRequest)> = Vec::new();
        let mut plans = Vec::new();

        for (source, target) in &file_map {
            let source_abs = rename::absolute_path(&root_dir, Path::new(source));
            let target_abs = rename::absolute_path(&root_dir, Path::new(target));
            refuse_module_boundary(&root_dir, &source_abs, &target_abs)?;

            if let Some(request) = build_symbol_rename_request(&root_dir, &source_abs, &target_abs)?
            {
                lsp_batch.push((source.clone(), target.clone(), request));
                continue;
            }
            plans.push(
                client
                    .plan_file_renames(
                        &[],
                        vec![(source.clone(), target.clone())],
                        Some(root_dir.as_path()),
                        "rust",
                        &["rs"],
                    )
                    .await?,
            );
        }
        let requests = symbol_requests(&root_dir, &lsp_batch);
        plans.extend(
            client
                .plan_symbol_renames(&[], Some(root_dir.as_path()), requests, "rust")
                .await?,
        );

        let mut preview = MovePreview {
            moves: file_map
                .iter()
                .map(|(from, to)| {
                    (
                        rename::absolute_path(&root_dir, Path::new(from)),
                        rename::absolute_path(&root_dir, Path::new(to)),
                    )
                })
                .collect(),
            ..Default::default()
        };
        summarize(&plans)?.apply_to(&mut preview, "rust-analyzer");
        Ok(preview)
    }
}

/// A file may be renamed inside its folder; moving it to another folder changes
/// the module tree, which is `move-module`'s job.
fn refuse_module_boundary(root_dir: &Path, source_abs: &Path, target_abs: &Path) -> Result<()> {
    if source_abs.parent() != target_abs.parent() {
        bail!(
            "Rust file move crosses a module boundary: {} -> {}. Use `refac move-module --project-path {} crate::<source> crate::<target>` so refac can move the complete logical module without introducing #[path] shims.",
            source_abs.display(),
            target_abs.display(),
            root_dir.display()
        );
    }
    Ok(())
}

/// One language-server request per module rename; a module that moves is
/// answered against the file it will be at.
fn symbol_requests(
    root_dir: &Path,
    batch: &[(String, String, RustSymbolRenameRequest)],
) -> Vec<SymbolRenameRequest> {
    batch
        .iter()
        .map(|(source, target, request)| {
            let source_abs = rename::absolute_path(root_dir, Path::new(source));
            let target_abs = rename::absolute_path(root_dir, Path::new(target));
            let mut pending_moves = std::collections::HashMap::new();
            pending_moves.insert(target_abs, source_abs);

            SymbolRenameRequest {
                document_path: request.document_path.clone(),
                position: request.position,
                new_name: request.new_name.clone(),
                pending_moves,
            }
        })
        .collect()
}
