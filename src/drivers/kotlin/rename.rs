//! Semantic symbol rename for Kotlin: the shared rename engine
//! (`lsp::rename`) with the JetBrains Kotlin language server. Kotlin adds what
//! the server never touches (Android XML that names a renamed class), lets a
//! class rename move its file, and expects the server to rewrite or drop the
//! symbol's own import line.

mod imports;

use super::android;
use super::android::class_renames;
use super::android::moved::MovedFile;
use super::android::stale;
use super::android::survey::survey;
use super::project::gradle_root;
use super::server::{self, KotlinServer};
use crate::drivers::lsp::rename::language::{EditedText, FollowUps, Language};
use crate::drivers::lsp::rename::plan::discover::RenamePlan;
use crate::drivers::lsp::rename::server::RenameServer;
use crate::drivers::lsp::rename::write::journal::FileWrite;
use crate::drivers::lsp::rename::{
    rename_symbol as rename_with, rename_symbols as rename_all_with,
};
use crate::drivers::preview::copy::{CopyPlan, ProjectRoot};
pub use crate::drivers::symbol::rename::{RenameReport, RenameRequest};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Words that cannot name a symbol (hard keywords).
const RESERVED_WORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

pub struct Kotlin;

pub async fn rename_symbol(request: RenameRequest) -> Result<RenameReport> {
    rename_with(&Kotlin, request).await
}

/// Several renames in one server session, which for Kotlin saves the half
/// minute the server needs to import the Gradle build for each of them.
pub async fn rename_all_symbols(requests: Vec<RenameRequest>) -> Result<Vec<RenameReport>> {
    if requests.len() > 1 && requests.iter().all(|request| request.dry_run) {
        return plan_on_copy(requests).await;
    }
    rename_all_with(&Kotlin, requests).await
}

/// A dry run of several renames. A Kotlin class rename moves its file and
/// edits Android XML, which the in-memory view that a dry run of the other
/// languages plans through does not follow, so the real batch runs on a
/// throw-away copy of the project and its reports are the plan. It costs a
/// Gradle import, as the real batch does, and the project is not touched.
async fn plan_on_copy(requests: Vec<RenameRequest>) -> Result<Vec<RenameReport>> {
    let root = ProjectRoot::at(Some(&requests[0].project_path))?;
    let copy = root.copy(&CopyPlan::default())?;
    let mut on_copy = Vec::new();
    for request in &requests {
        let mut moved = request.clone();
        moved.project_path = copy.path().to_path_buf();
        moved.file = root.relative(&request.file.to_string_lossy())?.into();
        moved.dry_run = false;
        on_copy.push(moved);
    }
    let reports = rename_all_with(&Kotlin, on_copy)
        .await
        .map_err(|error| anyhow::anyhow!(copy.about_the_project(&format!("{error:#}"))))?;
    Ok(reports
        .into_iter()
        .map(|report| RenameReport {
            dry_run: true,
            notes: report
                .notes
                .iter()
                .map(|note| copy.about_the_project(note))
                .collect(),
            ..report
        })
        .collect())
}

#[async_trait]
impl Language for Kotlin {
    fn name(&self) -> &'static str {
        "Kotlin"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["kt"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        RESERVED_WORDS
    }

    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        gradle_root(Some(project_path))
    }

    async fn start(&self, root: &Path, _file: &Path) -> Result<Box<dyn RenameServer>> {
        let install = server::locate()?;
        Ok(Box::new(KotlinServer::start(&install, root).await?))
    }

    fn exempt_edits(&self, edited: &EditedText) -> Vec<bool> {
        imports::exempt(edited)
    }

    /// A class rename renames its file.
    fn refuse_file_operations(&self) -> Option<&'static str> {
        None
    }

    /// What the rename implies beyond the server's edits: Android XML that
    /// names a renamed class, and old names left where refac does not edit.
    fn follow_ups(&self, root: &Path, plan: &RenamePlan) -> Result<FollowUps> {
        let moved: Vec<MovedFile> = plan
            .files
            .iter()
            .filter(|edited| edited.file.path.extension().and_then(|e| e.to_str()) == Some("kt"))
            .map(|edited| MovedFile {
                from: edited.file.path.clone(),
                to: new_path(&edited.file.path, plan),
                before: edited.file.before.clone(),
                after: edited.file.text.clone(),
            })
            .collect();
        let renames = class_renames::collect(&moved)?;
        let files = survey(root)?;
        let writes = android::plan(&files, &moved, &renames.classes)?;
        // Files already decided, so the scan reads them as they will be.
        let decided: Vec<FileWrite> = writes
            .iter()
            .cloned()
            .chain(plan.files.iter().map(|edited| FileWrite {
                path: edited.file.path.clone(),
                bytes: edited.file.bytes.clone(),
                changes: edited.edits.len(),
            }))
            .collect();
        let mut notes: Vec<String> = plan
            .moves
            .iter()
            .map(|(from, to)| {
                format!(
                    "{} is renamed to {} together with its class",
                    from.strip_prefix(root).unwrap_or(from).display(),
                    to.strip_prefix(root).unwrap_or(to).display()
                )
            })
            .collect();
        notes.extend(stale::scan(root, &files, &renames, &decided)?);
        Ok(FollowUps { writes, notes })
    }
}

fn new_path(path: &Path, plan: &RenamePlan) -> PathBuf {
    plan.moves
        .iter()
        .find(|(from, _)| from == path)
        .map_or_else(|| path.to_path_buf(), |(_, to)| to.clone())
}
