//! What a move and a dry run share before any driver is asked to do anything:
//! the request is validated, its paths are grouped by language, and every
//! driver the groups need is checked to be there.

use crate::drivers::RefactorDriver;
use crate::logic::RefactorRequest;
use crate::logic::grouping::referenced;
use crate::logic::grouping::route;
use crate::logic::grouping::typescript;
use crate::logic::grouping::unavailable;
use crate::logic::report::Pairs;
use crate::validation::initial_sanity_check;
use anyhow::{Result, bail};
use std::path::Path;

/// A request that passed every check that does not need the language tools to
/// do work.
pub(in crate::logic) struct Prepared<'a> {
    pub root: Option<&'a Path>,
    /// Per language, in language order: its pairs and the driver for them.
    pub groups: Vec<(String, Pairs, Box<dyn RefactorDriver>)>,
    /// Sources whose extension no driver handles.
    pub skipped: Vec<String>,
    pub typescript_source_count: usize,
    /// Pairs of the Markdown group that code uses: the TypeScript engine fixes
    /// that code before the Markdown driver moves the files.
    pub companion: Pairs,
}

pub(in crate::logic) async fn prepare(req: &RefactorRequest) -> Result<Prepared<'_>> {
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
    let mut batch_map: std::collections::HashMap<String, Pairs> = std::collections::HashMap::new();
    let mut skipped = Vec::new();

    for (src, tgt) in req.source_path.iter().zip(targets.iter()) {
        let root = req.project_path.as_deref().map(Path::new);
        let Some(lang) = route::language_of(src, root)? else {
            tracing::warn!("Skipping file with unsupported extension: {}", src);
            skipped.push(src.clone());
            continue;
        };
        batch_map
            .entry(lang.to_string())
            .or_default()
            .push((src.clone(), tgt.clone()));
    }

    // Pictures, pages and folders that code imports still belong to the
    // Markdown driver; the TypeScript engine fixes the imports.
    let root = req.project_path.as_deref().map(Path::new);
    let companion = referenced::used_pairs(&batch_map, root)?;

    // 3. Sorted for deterministic output order
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
    let mut groups = Vec::new();
    for (lang, files) in dispatch_order {
        let driver = route::driver_for(&lang)?;
        if !driver.check_availability().await? {
            bail!("{}", unavailable::message(&lang, root));
        }
        groups.push((lang, files, driver));
    }

    // The engine that fixes the code is a tool the request needs even when it
    // moves no TypeScript of its own.
    if !companion.is_empty() && !groups.iter().any(|(lang, ..)| lang == "typescript") {
        let driver = route::driver_for("typescript")?;
        if !driver.check_availability().await? {
            bail!(
                "Code in this project imports files you are moving, and the TypeScript engine fixes those imports. {}",
                unavailable::message("typescript", root)
            );
        }
    }

    Ok(Prepared {
        root,
        groups,
        skipped,
        typescript_source_count,
        companion,
    })
}
