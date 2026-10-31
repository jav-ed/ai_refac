use crate::logic::grouping::prepare::{Prepared, prepare};
use anyhow::{Result, bail};
use report::{FailedGroup, MoveOutcome, Pairs};
use std::collections::{BTreeMap, HashMap, HashSet};

mod code_references;
mod dry_run;
mod go_collaterals;
mod grouping;
pub mod kotlin_cost;
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

/// The request with an absolute project path. The drivers join the relative
/// source and target paths onto it, and a project path that is relative to the
/// current directory (`--project-path sub`) would be joined a second time by
/// the ones that start a server in that folder.
fn absolute_project(req: RefactorRequest) -> Result<RefactorRequest> {
    let project_path = match req.project_path.as_deref() {
        Some(root) => Some(std::path::absolute(root)?.to_string_lossy().into_owned()),
        None => None,
    };
    Ok(RefactorRequest {
        project_path,
        ..req
    })
}

/// Central entry point for handling refactor requests.
///
/// # Internal Docs
/// This function acts as the **Orchestrator**.
/// 1. Runs validation and groups the paths by language (`prepare`).
/// 2. Dispatches each group to its driver.
/// 3. Fixes the Markdown links to what the other languages moved.
pub async fn handle_refactor(req: RefactorRequest) -> Result<String> {
    let req = absolute_project(req)?;
    let Prepared {
        root,
        mut groups,
        skipped,
        typescript_source_count,
        companion,
    } = prepare(&req).await?;

    // The old paths, for the files that still name them after the move.
    let mentions = code_references::still_mentioned::collect_for(&groups, &companion, root);

    // Folders are gone once moved, so ask now for the Markdown link pass later.
    let directories = markdown_links::directories(&req.source_path, root);
    let mut moved: BTreeMap<String, Pairs> = BTreeMap::new();
    // What each driver reports beyond success, by language.
    let mut notes: HashMap<String, Vec<String>> = HashMap::new();
    let mut failed: Vec<FailedGroup> = Vec::new();

    // Code that imports the pages and pictures the Markdown driver is about to
    // move is fixed first. If the engine cannot, nothing has moved and the
    // whole request fails.
    if !companion.is_empty() {
        let fixed =
            code_references::imports::fix_before_moving(&mut groups, &companion, root).await?;
        if !fixed.typescript_files.is_empty() {
            moved.insert("typescript".to_string(), fixed.typescript_files);
        }
        notes
            .entry("markdown".to_string())
            .or_default()
            .extend(fixed.notes);
    }

    let single_kotlin = kotlin_cost::is_single_move(
        groups
            .iter()
            .map(|(lang, files, _)| (lang.as_str(), files.as_slice())),
        root,
    );

    for (lang, files, driver) in groups {
        let started = std::time::Instant::now();
        match driver.move_files_with_notes(files.clone(), root).await {
            Ok(mut driver_notes) => {
                if single_kotlin && lang == "kotlin" {
                    driver_notes.push(kotlin_cost::note("move", started.elapsed(), false));
                }
                notes.entry(lang.clone()).or_default().extend(driver_notes);
                moved.insert(lang, files);
            }
            Err(error) => {
                // The code was already fixed for the files this group did not move.
                let error = if lang == "markdown" && !companion.is_empty() {
                    error.context(
                        "The imports in code were already updated for these pages and pictures;                          restore them from version control or move the files by hand",
                    )
                } else {
                    error
                };
                failed.push(FailedGroup::new(lang, files, &error));
            }
        }
    }

    if let Some(mentions) = mentions
        && !moved.is_empty()
    {
        notes
            .entry("typescript".to_string())
            .or_default()
            .extend(mentions.scan(&HashSet::new()));
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
