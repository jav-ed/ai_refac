use super::{
    apply::{MovePlan, TextReplacement, append_replacement, apply_transaction, render_writes},
    layout,
    module_graph::{self, ResolvedModule},
    references, validation,
    workspace::SemanticWorkspace,
};
use anyhow::{Context, Result, bail};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub struct MoveModuleReport {
    pub moved_paths: usize,
    pub edited_files: usize,
}

pub fn move_module(root: &Path, source_path: &str, target_path: &str) -> Result<MoveModuleReport> {
    let source_segments = module_graph::parse_module_path(source_path)?;
    let target_segments = module_graph::parse_module_path(target_path)?;
    validation::validate_paths(&source_segments, &target_segments)?;

    let workspace = SemanticWorkspace::load(root)?;
    let source = module_graph::resolve_source(&workspace, &source_segments)?;
    module_graph::collect_subtree(&workspace, &source)?;
    if module_graph::find_module(&workspace, source.krate, &target_segments)?.is_some() {
        bail!("Target module `{target_path}` already exists");
    }

    let mut replacements =
        references::module_reference_edits(&workspace, &source, &target_segments)?;
    let module_files = layout::module_files(&source)?;
    if source_segments[..source_segments.len() - 1] != target_segments[..target_segments.len() - 1]
    {
        for (path, logical_module) in &module_files {
            let content = std::fs::read_to_string(path)?;
            replacements.extend(references::super_path_edits(
                path,
                &content,
                logical_module,
            )?);
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

    let reference_count = replacements.len();
    let writes = render_writes(replacements, new_files)?;
    let edited_files = writes.len();
    let moved_paths = moves.len();
    let plan = MovePlan {
        writes,
        moves,
        created_directories: created_directories.into_iter().collect(),
    };
    apply_transaction(plan, || {
        validation::validate_result(workspace.root(), &source_segments, &target_segments)
    })?;
    tracing::info!(
        reference_count,
        moved_paths,
        edited_files,
        "Rust semantic module move completed"
    );

    Ok(MoveModuleReport {
        moved_paths,
        edited_files,
    })
}

fn prepare_target_parent(
    workspace: &SemanticWorkspace,
    source: &ResolvedModule,
    target_parent: &[String],
    replacements: &mut Vec<TextReplacement>,
    new_files: &mut BTreeMap<PathBuf, String>,
    created_directories: &mut BTreeSet<PathBuf>,
) -> Result<PathBuf> {
    if let Some(parent) = module_graph::find_module(workspace, source.krate, target_parent)? {
        return Ok(parent.definition_file);
    }

    let mut existing_length = target_parent.len();
    let (mut parent_file, mut child_directory) = loop {
        if existing_length == 0 {
            let root_file = workspace.file_path(source.krate.root_file(workspace.database()))?;
            let child_directory = root_file
                .parent()
                .context("Cargo crate root has no parent directory")?
                .to_path_buf();
            break (root_file, child_directory);
        }
        if let Some(parent) =
            module_graph::find_module(workspace, source.krate, &target_parent[..existing_length])?
        {
            let child_directory = layout::child_module_directory(&parent.definition_file);
            break (parent.definition_file, child_directory);
        }
        existing_length -= 1;
    };

    for segment in &target_parent[existing_length..] {
        let new_directory = child_directory.join(segment);
        let new_module_file = new_directory.join("mod.rs");
        if new_module_file.exists() || new_directory.with_extension("rs").exists() {
            bail!(
                "Target parent `crate::{}` exists on disk but is not resolved by rust-analyzer",
                target_parent.join("::")
            );
        }

        let parent_content = new_files
            .get(&parent_file)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| std::fs::read_to_string(&parent_file))?;
        let visibility = if source.visibility.is_empty() {
            String::new()
        } else {
            format!("{} ", source.visibility)
        };
        replacements.push(append_replacement(
            &parent_file,
            &parent_content,
            &format!("{visibility}mod {segment};"),
        ));
        new_files.insert(new_module_file.clone(), String::new());
        layout::track_missing_directories(workspace.root(), &new_directory, created_directories)?;
        parent_file = new_module_file;
        child_directory = new_directory;
    }

    Ok(parent_file)
}

fn plan_declaration_edits(
    source: &ResolvedModule,
    target: &[String],
    target_parent_file: &Path,
    replacements: &mut Vec<TextReplacement>,
    new_files: &BTreeMap<PathBuf, String>,
) -> Result<()> {
    let target_name = target.last().context("Target module has no name")?;
    let source_parent = &source.segments[..source.segments.len() - 1];
    let target_parent = &target[..target.len() - 1];
    if source_parent == target_parent {
        replacements.push(TextReplacement::from_range(
            source.declaration_file.clone(),
            source.name_range,
            target_name.clone(),
        ));
        return Ok(());
    }

    let source_content = std::fs::read_to_string(&source.declaration_file)?;
    let (start, end) = full_line_range(
        &source_content,
        usize::from(source.declaration_range.start()),
        usize::from(source.declaration_range.end()),
    );
    replacements.push(TextReplacement {
        path: source.declaration_file.clone(),
        start,
        end,
        replacement: String::new(),
    });

    let local_name_start =
        usize::from(source.name_range.start() - source.declaration_range.start());
    let local_name_end = usize::from(source.name_range.end() - source.declaration_range.start());
    let mut declaration = source.declaration_text.clone();
    declaration.replace_range(local_name_start..local_name_end, target_name);
    let target_content = new_files
        .get(target_parent_file)
        .cloned()
        .map(Ok)
        .unwrap_or_else(|| std::fs::read_to_string(target_parent_file))?;
    replacements.push(append_replacement(
        target_parent_file,
        &target_content,
        declaration.trim(),
    ));
    Ok(())
}

fn full_line_range(content: &str, start: usize, end: usize) -> (usize, usize) {
    let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[end..]
        .find('\n')
        .map_or(content.len(), |offset| end + offset + 1);
    (line_start, line_end)
}
