//! A dry run for the backends that cannot plan a move without carrying it out:
//! Rope applies one move after the other and reads the moved files, and the
//! Kotlin server, Snapshot and Android layer need them in their new places.
//! A dry run of several Kotlin renames (a class rename moves its file) and
//! `move-module --dry-run --check` (a compile needs the moved files) work the
//! same way. The real operation runs on a throw-away copy of the project, and
//! what differs between the copy and the original is the preview. The original
//! is never touched.
//!
//! The copy holds what the project's `.gitignore` does not exclude (and never
//! `.git`), plus the few ignored files a tool needs to load the project. It is
//! limited in size (`REFAC_DRY_RUN_COPY_MAX_MB`, default 500) so that a dry run
//! cannot fill the disk; a project over the limit is an error that says so.

mod compare;
mod files;

use super::MovePreview;
use anyhow::{Context, Result, bail};
use compare::{compare, read_tree};
use files::{copy_project, limit_bytes};
use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Component, Path, PathBuf};

/// What a backend says about its copy.
#[derive(Default)]
pub struct CopyPlan<'a> {
    /// Names of folders (at any depth) whose content is the tool's own state
    /// and is left out of the comparison altogether, such as `.ropeproject`.
    pub tool_state: &'a [&'a str],
    /// Names of folders the tool writes build output into: a file created
    /// there is not reported, a file that was already there is compared like
    /// any other (a package may be called `build`).
    pub scratch: &'a [&'a str],
    /// File extensions (without the dot) the tool reads; the copy holds no
    /// other file. Empty copies every file the project does not ignore.
    pub only: &'a [&'a str],
    /// Names of folders (at any depth) the copy never holds, such as Cargo's
    /// `target`: build output the tool does not read.
    pub skip: &'a [&'a str],
}

/// The project folder as the caller named it, before anything is copied.
pub struct ProjectRoot {
    /// The real path of the folder.
    canonical: PathBuf,
    /// The folder as written, which differs when it is a link to the project.
    given: PathBuf,
}

/// A throw-away copy of a project. The folder is deleted when this is dropped.
pub struct ProjectCopy {
    dir: tempfile::TempDir,
    project: PathBuf,
    copied: BTreeSet<PathBuf>,
}

impl ProjectRoot {
    pub fn at(root_path: Option<&Path>) -> Result<Self> {
        let given = match root_path {
            Some(root) => std::path::absolute(root)?,
            None => std::env::current_dir()?,
        };
        let canonical = given
            .canonicalize()
            .with_context(|| format!("Cannot read the project folder {}", given.display()))?;
        Ok(Self { canonical, given })
    }

    pub fn path(&self) -> &Path {
        &self.canonical
    }

    /// `path` (relative to the project, or absolute below it) relative to the
    /// project. The copy has the same layout, so it is the path in the copy
    /// too. A path may be written through a link to the project folder.
    pub fn relative(&self, path: &str) -> Result<String> {
        let joined = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.canonical.join(path)
        };
        let normalized = normalize(&joined);
        let Some(relative) = [&self.canonical, &self.given]
            .into_iter()
            .find_map(|root| normalized.strip_prefix(root).ok())
        else {
            bail!(
                "A dry run copies the project, so every path must lie inside {}: {path} does not. Run the operation itself to reach outside the project.",
                self.canonical.display()
            );
        };
        Ok(relative.to_string_lossy().into_owned())
    }

    /// Copies the project into a folder of its own: what `plan.only` and
    /// `plan.skip` leave in, of what the project's `.gitignore` does not exclude.
    pub fn copy(&self, plan: &CopyPlan) -> Result<ProjectCopy> {
        let dir = tempfile::Builder::new()
            .prefix("refac-dry-run-")
            .tempdir()
            .context("Cannot create the folder for the dry-run copy")?;
        let copied = copy_project(&self.canonical, dir.path(), limit_bytes()?, plan)?;
        Ok(ProjectCopy {
            dir,
            project: self.canonical.clone(),
            copied,
        })
    }
}

impl ProjectCopy {
    /// The root of the copy, where the real operation runs.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// A message the real operation printed about the copy, as it reads about
    /// the project the user named.
    pub fn about_the_project(&self, text: &str) -> String {
        text.replace(
            &self.dir.path().display().to_string(),
            &self.project.display().to_string(),
        )
    }
}

/// Runs `run` (the backend's real move) on a copy of the project and returns
/// the difference. `run` gets the pairs relative to the copy and the copy's
/// root, and returns the notes the real move would print.
pub async fn preview_on_copy<F, Fut>(
    root_path: Option<&Path>,
    file_map: &[(String, String)],
    plan: CopyPlan<'_>,
    run: F,
) -> Result<MovePreview>
where
    F: FnOnce(Vec<(String, String)>, PathBuf) -> Fut,
    Fut: Future<Output = Result<Vec<String>>>,
{
    let root = ProjectRoot::at(root_path)?;
    let pairs: Vec<(String, String)> = file_map
        .iter()
        .map(|(source, target)| Ok((root.relative(source)?, root.relative(target)?)))
        .collect::<Result<_>>()?;
    let copy = root.copy(&plan)?;

    let notes = run(pairs.clone(), copy.path().to_path_buf()).await?;

    let after = read_tree(copy.path(), plan.tool_state)?;
    let mut preview = compare(root.path(), &copy.copied, &after, &pairs, &plan)?;
    preview.notes.splice(0..0, notes);
    Ok(preview)
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests;
