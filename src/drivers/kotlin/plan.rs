//! Turning the requested moves into the requests the Kotlin server can
//! answer. The server takes one target directory per request, cannot move and
//! rename in one step and does not move `.java` files, so this module checks
//! the request against those limits before any server is started.

use super::project::source_root;
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

/// A move changes the directory; a rename keeps it and changes the name. The
/// server answers a request that mixes both with `null`, so they never share one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Move,
    Rename,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub from: PathBuf,
    pub to: PathBuf,
    pub is_dir: bool,
}

/// Steps sent as one `workspace/willRenameFiles` request: same kind and the
/// same target directory.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub kind: Kind,
    pub steps: Vec<Step>,
}

#[derive(Debug, Default)]
pub struct MovePlan {
    pub groups: Vec<Group>,
    /// What the user asked for beyond a plain move, to report afterwards.
    pub notes: Vec<String>,
}

pub fn build(file_map: &[(String, String)], root: &Path) -> Result<MovePlan> {
    if file_map.is_empty() {
        bail!("No Kotlin move was requested");
    }
    let requested = file_map
        .iter()
        .map(|(source, target)| resolve_request(source, target, root))
        .collect::<Result<Vec<_>>>()?;
    check_overlaps(&requested)?;

    let mut plan = MovePlan::default();
    let mut first = Vec::new();
    let mut second = Vec::new();
    let mut taken: HashSet<PathBuf> = requested.iter().map(|step| step.to.clone()).collect();
    for step in requested {
        if step.from.parent() == step.to.parent() {
            first.push((Kind::Rename, step));
            continue;
        }
        if step.from.file_name() == step.to.file_name() {
            first.push((Kind::Move, step));
            continue;
        }
        // Move, then rename: the server needs two requests for this.
        let parent = step
            .to
            .parent()
            .context("A target without a parent directory")?;
        let name = step.from.file_name().context("A source without a name")?;
        let halfway = parent.join(name);
        if halfway.exists() || !taken.insert(halfway.clone()) {
            bail!(
                "Moving {} to {} needs the temporary name {}, which is taken. Move and rename it in two separate calls.",
                step.from.display(),
                step.to.display(),
                halfway.display()
            );
        }
        plan.notes.push(format!(
            "{} was moved and then renamed in two steps, because the Kotlin server cannot do both at once",
            step.from.display()
        ));
        first.push((
            Kind::Move,
            Step {
                from: step.from.clone(),
                to: halfway.clone(),
                is_dir: step.is_dir,
            },
        ));
        second.push((
            Kind::Rename,
            Step {
                from: halfway,
                to: step.to,
                is_dir: step.is_dir,
            },
        ));
    }
    plan.groups = group(first);
    plan.groups.extend(group(second));
    Ok(plan)
}

fn group(steps: Vec<(Kind, Step)>) -> Vec<Group> {
    let mut groups: Vec<(Option<PathBuf>, Group)> = Vec::new();
    for (kind, step) in steps {
        let target_dir = step.to.parent().map(Path::to_path_buf);
        let existing = groups
            .iter_mut()
            .find(|(dir, group)| *dir == target_dir && group.kind == kind);
        match existing {
            Some((_, group)) => group.steps.push(step),
            None => groups.push((
                target_dir,
                Group {
                    kind,
                    steps: vec![step],
                },
            )),
        }
    }
    groups.into_iter().map(|(_, group)| group).collect()
}

fn resolve_request(source: &str, target: &str, root: &Path) -> Result<Step> {
    let from = absolute(source, root);
    let from = from
        .canonicalize()
        .with_context(|| format!("Source does not exist: {}", from.display()))?;
    let to = canonical_target(&absolute(target, root))?;
    let is_dir = from.is_dir();

    if !is_dir {
        let extension = from.extension().and_then(|extension| extension.to_str());
        if extension == Some("java") {
            bail!(
                "{} is a Java file. The Kotlin server moves it without updating its package line, so refac refuses it. Move the Kotlin files and directories, or move Java with a Java-aware tool.",
                from.display()
            );
        }
        if extension != Some("kt") {
            bail!(
                "{} is not a Kotlin file (.kt) or a directory",
                from.display()
            );
        }
        if to.extension().and_then(|extension| extension.to_str()) != Some("kt") {
            bail!(
                "The target of {} must keep the .kt extension: {}",
                from.display(),
                to.display()
            );
        }
    }
    if is_dir && let Some(java) = first_java_file(&from)? {
        bail!(
            "{} contains Java sources ({}). The Kotlin server moves them without updating their package lines, so refac refuses it. Move the Kotlin files inside it one by one, or move the directory with a Java-aware tool.",
            from.display(),
            java.display()
        );
    }
    for path in [&from, &to] {
        let Some(sources) = source_root(path) else {
            bail!(
                "{} is not inside a source root (src/<source set>/kotlin or java). The Kotlin server derives packages from that position.",
                path.display()
            );
        };
        if *path == sources {
            bail!(
                "{} is a source root itself and cannot be moved or replaced",
                path.display()
            );
        }
    }
    // A move between modules or source sets changes what the code can see
    // (dependencies, test versus main), which no package edit can fix.
    if source_set(&from) != source_set(&to) {
        bail!(
            "{} and {} are in different modules or source sets. Only moves inside one source set (its kotlin and java folders) are supported.",
            from.display(),
            to.display()
        );
    }
    if to.exists() {
        bail!("Target already exists: {}", to.display());
    }
    Ok(Step { from, to, is_dir })
}

fn first_java_file(dir: &Path) -> Result<Option<PathBuf>> {
    for entry in WalkDir::new(dir) {
        let path = entry?.into_path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("java") {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// `<module>/src/<source set>`: the directory that holds the `kotlin` and
/// `java` folders of one compilation.
fn source_set(path: &Path) -> Option<PathBuf> {
    source_root(path)?.parent().map(Path::to_path_buf)
}

/// Requests must not depend on each other: no source inside another, no
/// duplicate, no target that is another request's source or lies inside one.
fn check_overlaps(steps: &[Step]) -> Result<()> {
    for (index, step) in steps.iter().enumerate() {
        if step.to.starts_with(&step.from) {
            bail!(
                "{} cannot be moved into itself ({})",
                step.from.display(),
                step.to.display()
            );
        }
        for other in &steps[index + 1..] {
            let related = step.from.starts_with(&other.from)
                || other.from.starts_with(&step.from)
                || step.to == other.to
                || step.to.starts_with(&other.from)
                || other.to.starts_with(&step.from);
            if related {
                bail!(
                    "The requests {} -> {} and {} -> {} overlap. Send them as separate calls.",
                    step.from.display(),
                    step.to.display(),
                    other.from.display(),
                    other.to.display()
                );
            }
        }
    }
    Ok(())
}

fn absolute(path: &str, root: &Path) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// A target does not exist yet, so resolve symlinks in its existing part only.
fn canonical_target(path: &Path) -> Result<PathBuf> {
    let mut tail = Vec::new();
    let mut existing = path;
    while !existing.exists() {
        let name = existing
            .file_name()
            .with_context(|| format!("Invalid target path {}", path.display()))?;
        tail.push(name.to_os_string());
        existing = existing
            .parent()
            .with_context(|| format!("Invalid target path {}", path.display()))?;
    }
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        bail!("Target paths must not contain . or .. : {}", path.display());
    }
    let mut resolved = existing.canonicalize()?;
    resolved.extend(tail.iter().rev());
    Ok(resolved)
}

#[cfg(test)]
mod tests;
