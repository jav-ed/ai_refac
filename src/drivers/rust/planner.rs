use super::{
    analysis::{lockfile::NewLockfiles, module_graph, workspace::SemanticWorkspace},
    edits::{macro_paths, macro_references, references, super_paths},
    target_parent::{plan_declaration_edits, prepare_target_parent},
    transaction::{
        apply::{MovePlan, TextReplacement, apply_transaction, render_writes},
        layout, validation,
    },
};
use anyhow::{Context, Result, bail};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// How far `move_module` goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveMode {
    /// Move the module, check that the workspace still compiles, and put
    /// everything back when it does not.
    Apply,
    /// Plan the move and write nothing; the compile check is not made.
    Plan,
    /// Plan the move by carrying it out on a copy of the workspace, compile the
    /// copy, and write nothing to the project.
    PlanAndCompile,
}

pub struct MoveModuleReport {
    /// True when nothing was written to the project.
    pub dry_run: bool,
    /// True when the moved workspace passed `cargo check`: always after a real
    /// move, and in a dry run only when `MoveMode::PlanAndCompile` asked for it.
    pub compiled: bool,
    pub moved_paths: usize,
    pub edited_files: usize,
    /// Text edits in total, and per file (paths relative to the workspace).
    pub edits: usize,
    pub files: Vec<(PathBuf, usize)>,
    /// Files and folders that move, relative to the workspace.
    pub moves: Vec<(PathBuf, PathBuf)>,
}

pub fn move_module(
    root: &Path,
    source_path: &str,
    target_path: &str,
    mode: MoveMode,
) -> Result<MoveModuleReport> {
    if mode == MoveMode::PlanAndCompile {
        return super::compile_check::plan_on_copy(root, source_path, target_path);
    }
    let dry_run = mode == MoveMode::Plan;
    let source_segments = module_graph::parse_module_path(source_path)?;
    let target_segments = module_graph::parse_module_path(target_path)?;
    validation::validate_paths(&source_segments, &target_segments)?;

    // Cargo writes a missing Cargo.lock while the workspace loads; a dry run
    // and a move that fails leave the folder as they found it.
    let root = root
        .canonicalize()
        .with_context(|| format!("Could not resolve Cargo workspace at {}", root.display()))?;
    let mut lockfiles = NewLockfiles::watch(&root);
    let workspace = SemanticWorkspace::load(&root)?;
    let source = module_graph::resolve_source(&workspace, &source_segments)?;
    module_graph::collect_subtree(&workspace, &source)?;
    if module_graph::find_module(&workspace, source.krate, &target_segments)?.is_some() {
        bail!("Target module `{target_path}` already exists");
    }

    let mut replacements =
        references::module_reference_edits(&workspace, &source, &target_segments)?;
    // Paths inside macro arguments, which the reference search does not list
    // unless it can expand the macro: first by name resolution, then (for a
    // scope rust-analyzer cannot give) the `crate::` spelling by its tokens.
    add_without_overlap(
        &mut replacements,
        macro_references::macro_reference_edits(&workspace, &source, &target_segments)?,
    );
    for (_, file) in module_graph::crate_source_files(&workspace, source.krate)? {
        let content = std::fs::read_to_string(&file)?;
        add_without_overlap(
            &mut replacements,
            macro_paths::macro_path_edits(&file, &content, &source_segments, &target_segments),
        );
    }
    let module_files = layout::module_files(&source)?;
    if source_segments[..source_segments.len() - 1] != target_segments[..target_segments.len() - 1]
    {
        for (path, logical_module) in &module_files {
            let content = std::fs::read_to_string(path)?;
            replacements.extend(super_paths::super_path_edits(
                path,
                &content,
                logical_module,
                &source_segments,
            )?);
            add_without_overlap(
                &mut replacements,
                super_paths::super_macro_path_edits(
                    path,
                    &content,
                    logical_module,
                    &source_segments,
                ),
            );
        }
    }

    let mut new_files = BTreeMap::new();
    let mut created_directories = BTreeSet::new();
    let target_parent = prepare_target_parent(
        &workspace,
        &source,
        &target_segments[..target_segments.len() - 1],
        &mut replacements,
        &mut new_files,
        &mut created_directories,
    )?;
    plan_declaration_edits(
        &source,
        &target_segments,
        &target_parent,
        &mut replacements,
        &new_files,
    )?;

    let moves = layout::physical_moves(&source, &target_parent, target_segments.last().unwrap())?;
    for movement in &moves {
        if movement.target.exists() {
            bail!(
                "Rust module move target already exists: {}",
                movement.target.display()
            );
        }
        let parent = movement
            .target
            .parent()
            .context("Move target has no parent directory")?;
        layout::track_missing_directories(workspace.root(), parent, &mut created_directories)?;
    }

    let edits = replacements.len();
    let files = edits_per_file(&replacements, workspace.root());
    let relative = |path: &Path| {
        path.strip_prefix(workspace.root())
            .unwrap_or(path)
            .to_path_buf()
    };
    let planned_moves = moves
        .iter()
        .map(|movement| (relative(&movement.source), relative(&movement.target)))
        .collect::<Vec<_>>();
    let writes = render_writes(replacements, new_files)?;
    let edited_files = writes.len();
    let moved_paths = moves.len();
    if !dry_run {
        let plan = MovePlan {
            writes,
            moves,
            created_directories: created_directories.into_iter().collect(),
        };
        apply_transaction(plan, || {
            validation::validate_result(workspace.root(), &source_segments, &target_segments)
        })?;
        lockfiles.keep();
    }
    tracing::info!(
        edits,
        moved_paths,
        edited_files,
        dry_run,
        "Rust semantic module move completed"
    );

    Ok(MoveModuleReport {
        dry_run,
        compiled: !dry_run,
        moved_paths,
        edited_files,
        edits,
        files,
        moves: planned_moves,
    })
}

/// How many edits each file gets, by path relative to the workspace root.
fn edits_per_file(replacements: &[TextReplacement], root: &Path) -> Vec<(PathBuf, usize)> {
    let mut counts = BTreeMap::<PathBuf, usize>::new();
    for edit in replacements {
        let path = edit.path.strip_prefix(root).unwrap_or(&edit.path);
        *counts.entry(path.to_path_buf()).or_default() += 1;
    }
    counts.into_iter().collect()
}

/// Adds the edits that do not touch text an earlier edit already rewrites.
fn add_without_overlap(
    replacements: &mut Vec<TextReplacement>,
    edits: impl IntoIterator<Item = TextReplacement>,
) {
    for edit in edits {
        let overlaps = replacements.iter().any(|existing| {
            existing.path == edit.path && existing.start < edit.end && edit.start < existing.end
        });
        if !overlaps {
            replacements.push(edit);
        }
    }
}
