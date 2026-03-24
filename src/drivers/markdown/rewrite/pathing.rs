use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use pathdiff::diff_paths;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(super) struct ResolvedMove {
    pub source_abs: PathBuf,
    pub target_abs: PathBuf,
}

pub(super) fn resolve_moves(
    file_map: &[(String, String)],
    root_path: Option<&Path>,
) -> Result<Vec<ResolvedMove>> {
    let mut resolved = Vec::with_capacity(file_map.len());
    let mut sources = HashSet::new();
    let mut targets = HashSet::new();

    for (source, target) in file_map {
        let source_abs = resolve_project_path(source, root_path)?;
        let target_abs = resolve_project_path(target, root_path)?;

        if source_abs.extension().and_then(|ext| ext.to_str()) != Some("md") {
            bail!(
                "Markdown driver only supports .md files: {}",
                source_abs.display()
            );
        }
        if target_abs.extension().and_then(|ext| ext.to_str()) != Some("md") {
            bail!(
                "Markdown driver target must end with .md: {}",
                target_abs.display()
            );
        }
        if !source_abs.is_file() {
            bail!(
                "Markdown source path must be an existing file: {}",
                source_abs.display()
            );
        }
        if source_abs != target_abs && target_abs.exists() {
            bail!("Markdown target already exists: {}", target_abs.display());
        }
        if !sources.insert(source_abs.clone()) {
            bail!(
                "Duplicate markdown source path in batch: {}",
                source_abs.display()
            );
        }
        if !targets.insert(target_abs.clone()) {
            bail!(
                "Duplicate markdown target path in batch: {}",
                target_abs.display()
            );
        }

        resolved.push(ResolvedMove {
            source_abs,
            target_abs,
        });
    }

    Ok(resolved)
}

pub(super) fn determine_workspace_root(
    resolved_moves: &[ResolvedMove],
    root_path: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(root) = root_path {
        return Ok(normalize_absolute_path(root)?);
    }

    let mut roots = resolved_moves
        .iter()
        .flat_map(|entry| [entry.source_abs.as_path(), entry.target_abs.as_path()])
        .filter_map(|path| path.parent().map(Path::to_path_buf));

    let Some(first) = roots.next() else {
        bail!("Cannot determine markdown workspace root from an empty move set");
    };

    Ok(roots.fold(first, common_prefix_dir))
}

pub(super) fn collect_markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("md")
        {
            files.push(normalize_path(path.to_path_buf()));
        }
    }

    Ok(files)
}

pub(super) async fn read_markdown_files(paths: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
    let mut contents = HashMap::with_capacity(paths.len());
    for path in paths {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read markdown file {}", path.display()))?;
        contents.insert(path.clone(), content);
    }
    Ok(contents)
}

fn resolve_project_path(path: &str, root_path: Option<&Path>) -> Result<PathBuf> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Ok(normalize_path(candidate.to_path_buf()));
    }

    let base = if let Some(root) = root_path {
        root.to_path_buf()
    } else {
        std::env::current_dir().context("Failed to read current directory")?
    };

    Ok(normalize_path(base.join(candidate)))
}

fn normalize_absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(normalize_path(path.to_path_buf()))
    } else {
        let cwd = std::env::current_dir().context("Failed to read current directory")?;
        Ok(normalize_path(cwd.join(path)))
    }
}

pub(super) fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }

    normalized
}

fn common_prefix_dir(left: PathBuf, right: PathBuf) -> PathBuf {
    let left_components: Vec<_> = left.components().collect();
    let right_components: Vec<_> = right.components().collect();
    let shared = left_components
        .iter()
        .zip(right_components.iter())
        .take_while(|(l, r)| l == r)
        .count();

    let mut root = PathBuf::new();
    for component in left_components.into_iter().take(shared) {
        root.push(component.as_os_str());
    }
    root
}

pub(super) fn relative_link_from(from_dir: &Path, to_path: &Path) -> Result<String> {
    let relative = diff_paths(to_path, from_dir).ok_or_else(|| {
        anyhow!(
            "Failed to compute relative markdown path from {} to {}",
            from_dir.display(),
            to_path.display()
        )
    })?;

    let mut display = relative.to_string_lossy().replace('\\', "/");
    if display.is_empty() {
        bail!(
            "Cannot emit an empty markdown link path from {} to {}",
            from_dir.display(),
            to_path.display()
        );
    }

    if !display.starts_with("./") && !display.starts_with("../") {
        display = format!("./{display}");
    }

    Ok(display)
}
