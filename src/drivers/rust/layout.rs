use super::{apply::PlannedMove, module_graph::ResolvedModule};
use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn physical_moves(
    source: &ResolvedModule,
    target_parent_file: &Path,
    target_name: &str,
) -> Result<Vec<PlannedMove>> {
    let child_directory = child_module_directory(target_parent_file);
    if source.is_mod_rs {
        let source_directory = source
            .definition_file
            .parent()
            .context("mod.rs module has no directory")?
            .to_path_buf();
        return Ok(vec![PlannedMove {
            source: source_directory,
            target: child_directory.join(target_name),
        }]);
    }

    let mut moves = vec![PlannedMove {
        source: source.definition_file.clone(),
        target: child_directory.join(target_name).with_extension("rs"),
    }];
    let companion = source.definition_file.with_extension("");
    if companion.is_dir() {
        moves.push(PlannedMove {
            source: companion,
            target: child_directory.join(target_name),
        });
    }
    Ok(moves)
}

pub fn module_files(source: &ResolvedModule) -> Result<Vec<(PathBuf, Vec<String>)>> {
    let base_directory = if source.is_mod_rs {
        source.definition_file.parent().unwrap().to_path_buf()
    } else {
        source.definition_file.with_extension("")
    };
    let mut files = vec![(source.definition_file.clone(), source.segments.clone())];
    if !base_directory.is_dir() {
        return Ok(files);
    }

    for entry in WalkDir::new(&base_directory)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("rs")
            || path == source.definition_file
        {
            continue;
        }
        let relative = path.strip_prefix(&base_directory)?;
        let mut logical = source.segments.clone();
        let mut components = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        if components.last().is_some_and(|name| name == "mod.rs") {
            components.pop();
        } else if let Some(last) = components.last_mut() {
            *last = last.strip_suffix(".rs").unwrap().to_string();
        }
        logical.extend(components);
        files.push((path.to_path_buf(), logical));
    }
    Ok(files)
}

pub fn child_module_directory(module_file: &Path) -> PathBuf {
    if module_file.file_name().and_then(|name| name.to_str()) == Some("mod.rs")
        || matches!(
            module_file.file_name().and_then(|name| name.to_str()),
            Some("lib.rs" | "main.rs")
        )
    {
        module_file.parent().unwrap().to_path_buf()
    } else {
        module_file.with_extension("")
    }
}

pub fn track_missing_directories(
    root: &Path,
    directory: &Path,
    created: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !directory.starts_with(root) {
        bail!(
            "Refusing to create a Rust module directory outside {}",
            root.display()
        );
    }
    let mut current = directory.to_path_buf();
    while !current.exists() && current.starts_with(root) {
        created.insert(current.clone());
        current = current
            .parent()
            .context("Directory has no parent")?
            .to_path_buf();
    }
    Ok(())
}
