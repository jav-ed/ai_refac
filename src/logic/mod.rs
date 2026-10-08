use crate::logic::grouping::prepare::{Prepared, prepare};
use anyhow::{Result, bail};
use report::{FailedGroup, MoveOutcome, Pairs};
use std::collections::{BTreeMap, HashMap};

mod dry_run;
mod go_collaterals;
mod grouping;
mod markdown_links;
pub mod rename;
mod report;

pub use dry_run::{DryRun, plan_refactor};

/// Parameters for a refactoring request.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RefactorRequest {
    #[schemars(
        description = "List of source paths. Use paths relative to project_path (recommended). Example: `Src/Features/Auth/login_Service.ts`."
    )]
    pub source_path: Vec<String>,
    #[schemars(
        description = "List of target paths (1:1 mapping with source_path). Use the same path style as source_path and keep them relative to project_path."
    )]
    pub target_path: Option<Vec<String>>,
    #[schemars(description = "Type of operation (currently only 'move' is supported)")]
    pub operation: String,
    #[schemars(
        description = "Absolute path to the language package root (not monorepo root). For TypeScript/JS, this should usually be the folder containing the relevant `tsconfig.json`."
    )]
    pub project_path: Option<String>,
}

/// Central entry point for handling refactor requests.
///
/// # Internal Docs
/// This function acts as the **Orchestrator**.
/// 1. Runs validation and groups the paths by language (`prepare`).
/// 2. Dispatches each group to its driver.
/// 3. Fixes the Markdown links to what the other languages moved.
pub async fn handle_refactor(req: RefactorRequest) -> Result<String> {
    let Prepared {
        root,
        groups,
        skipped,
        typescript_source_count,
    } = prepare(&req).await?;

    // Folders are gone once moved, so ask now for the Markdown link pass later.
    let directories = markdown_links::directories(&req.source_path, root);
    let mut moved: BTreeMap<String, Pairs> = BTreeMap::new();
    // What each driver reports beyond success, by language.
    let mut notes: HashMap<String, Vec<String>> = HashMap::new();
    let mut failed: Vec<FailedGroup> = Vec::new();

    for (lang, files, driver) in groups {
        match driver.move_files_with_notes(files.clone(), root).await {
            Ok(driver_notes) => {
                notes.insert(lang.clone(), driver_notes);
                moved.insert(lang, files);
            }
            Err(error) => failed.push(FailedGroup {
                lang,
                files,
                error: error.to_string(),
            }),
        }
    }

    // Markdown links to what the other languages moved.
    let link_notes = if moved.is_empty() {
        None
    } else {
        markdown_links::update(&moved, &directories, root).await?
    };

    // Build response. A request that was not carried out in full is an
    // error with the whole report as its text, so the exit code says so.
    let outcome = MoveOutcome {
        root,
        moved,
        notes,
        link_notes,
        failed,
        skipped,
        typescript_source_count,
    };
    let report = outcome.render();
    if outcome.is_complete() {
        Ok(report)
    } else {
        bail!("{report}")
    }
}
