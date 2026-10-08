use super::{MovePreview, RefactorDriver, complete_filesystem_moves};
use crate::drivers::lsp::client::{LspClient, SymbolRenameRequest, summarize};
use crate::servers;
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

mod requests;

use requests::{
    GoPackageRenameRequest, build_go_package_rename_request, build_go_post_lsp_source_path,
    resolve_abs_path, resolve_root_dir,
};

#[derive(Default)]
pub struct GoDriver;

impl GoDriver {
    pub fn new() -> Self {
        // gopls might be in PATH or in GOPATH/bin
        // We'll try to find it dynamically or default to "gopls"
        Self
    }
}

#[async_trait]
impl RefactorDriver for GoDriver {
    fn lang(&self) -> &str {
        "go"
    }

    async fn check_availability(&self) -> Result<bool> {
        Ok(servers::executable("go", &std::env::current_dir()?).is_ok())
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let root_dir = resolve_root_dir(root_path)?;
        let binary = servers::executable("go", &root_dir)?;
        let client = LspClient::new(&binary.to_string_lossy());

        let lsp_package_requests = package_requests(&root_dir, &file_map)?;

        // Pass 2: run all cross-dir package renames in one gopls session.
        // Maps source_dir → Vec<(old_abs, new_abs)> of file renames gopls reported.
        let mut package_file_renames: std::collections::HashMap<PathBuf, Vec<(PathBuf, PathBuf)>> =
            std::collections::HashMap::new();

        if !lsp_package_requests.is_empty() {
            let all_file_renames = client
                .initialize_and_rename_symbols_batch(
                    &[],
                    Some(root_dir.as_path()),
                    symbol_requests(&root_dir, &lsp_package_requests),
                    "go",
                )
                .await?;

            for (i, (source_dir, _, _)) in lsp_package_requests.iter().enumerate() {
                if let Some(renames) = all_file_renames.get(i) {
                    package_file_renames.insert(source_dir.clone(), renames.clone());
                }
            }
        }

        // Pass 3: apply filesystem moves per file.
        for (source, target) in &file_map {
            let single_move = vec![(source.clone(), target.clone())];
            let source_abs = resolve_abs_path(&root_dir, Path::new(source));
            let target_abs = resolve_abs_path(&root_dir, Path::new(target));
            let source_dir = source_abs.parent().map(|p| p.to_path_buf());

            // Find where gopls actually placed our source file via a RenameFile
            // resource op returned in the workspace edit.  Fall back to the
            // heuristic (target_dir/source_filename) only when the LSP did not
            // report an explicit rename for this file.
            let lsp_moved_to = source_dir
                .as_ref()
                .and_then(|dir| package_file_renames.get(dir))
                .and_then(|renames| renames.iter().find(|(from, _)| from == &source_abs))
                .map(|(_, to)| to.clone());

            let effective_source = lsp_moved_to.or_else(|| {
                build_go_post_lsp_source_path(
                    &source_abs,
                    &target_abs,
                    source_abs.parent() != target_abs.parent(),
                )
            });

            if let Some(lsp_source_abs) = effective_source {
                complete_go_filesystem_move(&lsp_source_abs, &target_abs).await?;
            } else {
                complete_filesystem_moves(&single_move, Some(root_dir.as_path())).await?;
            }
        }

        Ok(())
    }

    /// The package renames gopls proposes, each answered against the files as
    /// they are now. A move to another directory renames the whole package, so
    /// the files that travel with it are listed too.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        let root_dir = resolve_root_dir(root_path)?;
        let binary = servers::executable("go", &root_dir)?;
        let client = LspClient::new(&binary.to_string_lossy());

        let requests = package_requests(&root_dir, &file_map)?;
        let plans = client
            .plan_symbol_renames(
                &[],
                Some(root_dir.as_path()),
                symbol_requests(&root_dir, &requests),
                "go",
            )
            .await?;

        let mut preview = MovePreview {
            moves: file_map
                .iter()
                .map(|(from, to)| {
                    (
                        resolve_abs_path(&root_dir, Path::new(from)),
                        resolve_abs_path(&root_dir, Path::new(to)),
                    )
                })
                .collect(),
            ..Default::default()
        };
        let requested = preview.moves.len();
        summarize(&plans)?.apply_to(&mut preview, "gopls");
        if preview.moves.len() > requested {
            let travelling: Vec<String> = preview.moves[requested..]
                .iter()
                .map(|(from, _)| from.display().to_string())
                .collect();
            preview.notes.push(format!(
                "Go moves entire packages. These files would also be relocated as part of the package rename: {}.",
                travelling.join(", ")
            ));
        }
        Ok(preview)
    }
}

/// Pass 1: one request per unique source package (directory). Go's
/// package-per-directory model means gopls renames the entire package when any
/// file moves cross-directory, so one representative rename per unique source
/// dir is sufficient. A source dir is only marked "seen" when a request was
/// obtained for it, so same-dir moves (which return None from
/// `build_go_package_rename_request`) do not suppress a later cross-dir move
/// from the same directory. The tuple is (source_dir, target_abs, request).
fn package_requests(
    root_dir: &Path,
    file_map: &[(String, String)],
) -> Result<Vec<(PathBuf, PathBuf, GoPackageRenameRequest)>> {
    let mut seen_source_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut requests = Vec::new();

    for (source, target) in file_map {
        let source_abs = resolve_abs_path(root_dir, Path::new(source));
        let target_abs = resolve_abs_path(root_dir, Path::new(target));
        let source_dir = match source_abs.parent() {
            Some(d) => d.to_path_buf(),
            None => continue,
        };

        if seen_source_dirs.contains(&source_dir) {
            continue;
        }

        if let Some(request) = build_go_package_rename_request(root_dir, &source_abs, &target_abs)?
        {
            seen_source_dirs.insert(source_dir.clone());
            requests.push((source_dir, target_abs, request));
        }
    }
    Ok(requests)
}

fn symbol_requests(
    root_dir: &Path,
    requests: &[(PathBuf, PathBuf, GoPackageRenameRequest)],
) -> Vec<SymbolRenameRequest> {
    requests
        .iter()
        .map(|(_, target_abs, request)| {
            let source_abs = resolve_abs_path(root_dir, &request.document_path);
            let mut pending_moves = std::collections::HashMap::new();
            pending_moves.insert(target_abs.clone(), source_abs);
            SymbolRenameRequest {
                document_path: request.document_path.clone(),
                position: request.position,
                new_name: request.new_name.clone(),
                pending_moves,
            }
        })
        .collect()
}

async fn complete_go_filesystem_move(source_abs: &Path, target_abs: &Path) -> Result<()> {
    if source_abs == target_abs {
        return Ok(());
    }

    if target_abs.exists() && !source_abs.exists() {
        return Ok(());
    }

    if let Some(parent) = target_abs.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match tokio::fs::rename(source_abs, target_abs).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && target_abs.exists() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests;
