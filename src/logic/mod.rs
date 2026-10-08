use crate::validation::initial_sanity_check;
use anyhow::{Result, bail};
use report::{FailedGroup, MoveOutcome, Pairs};
use std::collections::{BTreeMap, HashMap};

mod go_collaterals;
mod markdown_links;
pub mod rename;
mod report;
mod route;
mod typescript;
mod unavailable;

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
/// 1. Runs validation.
/// 2. Determines which driver to use (TODO).
/// 3. Dispatches the request.
pub async fn handle_refactor(req: RefactorRequest) -> Result<String> {
    // 1. Validation
    initial_sanity_check(
        &req.source_path,
        &req.operation,
        req.target_path.as_ref(),
        req.project_path.as_deref(),
    )?;

    // 2. Group files by language
    let targets = req
        .target_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Target path required for move"))?;

    // Map: Language -> Vec<(Source, Target)>
    let mut batch_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut skipped_files = Vec::new();

    for (src, tgt) in req.source_path.iter().zip(targets.iter()) {
        let root = req.project_path.as_deref().map(std::path::Path::new);
        let Some(lang) = route::language_of(src, root)? else {
            tracing::warn!("Skipping file with unsupported extension: {}", src);
            skipped_files.push(src.clone());
            continue;
        };
        batch_map
            .entry(lang.to_string())
            .or_default()
            .push((src.clone(), tgt.clone()));
    }

    // 3. Dispatch Batches — sorted for deterministic output order
    let root = req.project_path.as_ref().map(std::path::Path::new);
    // Folders are gone once moved, so ask now for the Markdown link pass later.
    let directories = markdown_links::directories(&req.source_path, root);
    let mut moved: BTreeMap<String, Pairs> = BTreeMap::new();
    // What each driver reports beyond success, by language.
    let mut notes: HashMap<String, Vec<String>> = HashMap::new();
    let mut failed: Vec<FailedGroup> = Vec::new();

    let mut dispatch_order: Vec<(String, Pairs)> = batch_map.into_iter().collect();
    dispatch_order.sort_by(|a, b| a.0.cmp(&b.0));

    // Enforce the documented limit before any language batch mutates the project.
    let typescript_source_count = dispatch_order
        .iter()
        .find(|(lang, _)| lang == "typescript")
        .map(|(_, files)| typescript::count_source_files(files, root))
        .transpose()?
        .unwrap_or(0);
    if typescript_source_count > typescript::MAX_FILES_PER_MOVE {
        bail!(
            "TypeScript/JavaScript move contains {} source files; the maximum is {}. Split the move into smaller batches.",
            typescript_source_count,
            typescript::MAX_FILES_PER_MOVE
        );
    }

    // Every language server and tool a group needs must be there before the
    // first group moves, or a missing one would stop the command half way.
    let mut drivers = Vec::new();
    for (lang, files) in dispatch_order {
        let driver = route::driver_for(&lang)?;
        if !driver.check_availability().await? {
            bail!("{}", unavailable::message(&lang, root));
        }
        drivers.push((lang, files, driver));
    }

    for (lang, files, driver) in drivers {
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

    // 4. Build response. A request that was not carried out in full is an
    // error with the whole report as its text, so the exit code says so.
    let outcome = MoveOutcome {
        root,
        moved,
        notes,
        link_notes,
        failed,
        skipped: skipped_files,
        typescript_source_count,
    };
    let report = outcome.render();
    if outcome.is_complete() {
        Ok(report)
    } else {
        bail!("{report}")
    }
}
