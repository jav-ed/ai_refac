//! Markdown and the files Markdown points at.
//!
//! Two jobs share one engine. Moving Markdown files, asset files (images,
//! PDFs, ...), and folders of them rewrites every link that follows them; and
//! after another language's backend has moved files, `update_links_after_moves`
//! fixes the Markdown links to them. Links are found by a CommonMark parser
//! (`parser/`), changed only in their destination, and written the way the
//! author wrote them (`href.rs`).

use super::RefactorDriver;
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

mod apply;
mod documents;
mod href;
mod moves;
mod parser;
mod plan;
mod rewrite;
mod workspace;

/// Markdown files, the assets they point at, and folders of both. One driver
/// moves them all so that a request with a page and its image is planned as a
/// whole.
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
        documents::move_documents(file_map, root_path)
            .await
            .map(|_| ())
    }

    async fn move_files_with_notes(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&Path>,
    ) -> Result<Vec<String>> {
        documents::move_documents(file_map, root_path).await
    }
}

/// A path another backend moved: where it was, where it is, and whether it was
/// a folder.
#[derive(Debug, Clone)]
pub struct MovedPath {
    pub from: PathBuf,
    pub to: PathBuf,
    pub is_dir: bool,
}

/// What `update_links_after_moves` did, as sentences for the response.
#[derive(Debug, Default)]
pub struct LinkUpdate {
    pub links_updated: usize,
    pub notes: Vec<String>,
}

/// Fix the Markdown links that point at files another backend has moved.
/// Relative paths are taken from `root` (the project path), or from the working
/// directory when there is none; without a project path only the Markdown files
/// below the folder all moves share are checked. A link is only changed when
/// its old target is gone and the new one exists.
pub async fn update_links_after_moves(
    moved: Vec<MovedPath>,
    root: Option<&Path>,
) -> Result<LinkUpdate> {
    let base = documents::base_directory(root)?;
    let moved = moved
        .into_iter()
        .map(|entry| moves::Move {
            from: moves::absolute(&entry.from, &base),
            to: moves::absolute(&entry.to, &base),
            is_dir: entry.is_dir,
        })
        .collect();
    let moves = moves::MoveSet::new(moved)?;
    let scan_root = documents::scan_root(root, &base, &moves);

    let plan = plan::plan(moves, &scan_root, plan::When::AfterMoving).await?;
    apply::apply(&plan).await?;
    Ok(LinkUpdate {
        links_updated: plan.links_updated,
        notes: documents::notes(&plan, &scan_root),
    })
}
