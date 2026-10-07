//! Markdown links to files another language's backend has moved.
//!
//! A README links to `src/app.py` and a guide to `docs/diagram.png`; when the
//! Python backend moves `app.py` the link would be dead. After the language
//! batches are done, one pass over the project's Markdown files fixes the links
//! to everything that moved. The Markdown batch (Markdown files, assets, document
//! folders) is not part of it: that driver updates links itself.

use crate::drivers::markdown::{MovedPath, update_links_after_moves};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Languages whose batches already update the Markdown links themselves.
const LINK_AWARE: [&str; 1] = ["markdown"];

/// The requested sources that are folders. Asked before anything moves,
/// because afterwards the folder is gone.
pub fn directories(sources: &[String], root: Option<&Path>) -> HashSet<String> {
    sources
        .iter()
        .filter(|source| {
            let path = Path::new(source.as_str());
            match root {
                Some(root) if !path.is_absolute() => root.join(path).is_dir(),
                _ => path.is_dir(),
            }
        })
        .cloned()
        .collect()
}

/// Fix the links to the files `successful` batches moved. The text for the
/// response, or `None` when no batch needs it.
pub async fn update(
    successful: &HashMap<String, Vec<(String, String)>>,
    directories: &HashSet<String>,
    root: Option<&Path>,
) -> Result<Option<Vec<String>>> {
    let mut moved: Vec<MovedPath> = successful
        .iter()
        .filter(|(lang, _)| !LINK_AWARE.contains(&lang.as_str()))
        .flat_map(|(_, files)| files)
        .map(|(source, target)| MovedPath {
            from: source.into(),
            to: target.into(),
            is_dir: directories.contains(source),
        })
        .collect();
    if moved.is_empty() {
        return Ok(None);
    }
    moved.sort_by(|a, b| a.from.cmp(&b.from));

    // The files are already moved and cannot be put back from here, so say so.
    let update = update_links_after_moves(moved, root).await.context(
        "The files were moved, but updating the Markdown links that point at them failed. \
         The error is below; fix the links by hand or move the files back",
    )?;
    Ok(Some(update.notes))
}
