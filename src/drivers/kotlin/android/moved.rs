//! Which files a move carried to which place, and what they said before and
//! after. The Android layer and the stale-reference scan work from this, not
//! from the plan: a file renamed in a second step still maps from its original
//! place to its final one.

use crate::drivers::kotlin::plan::{MovePlan, Step};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A Kotlin file that ended up somewhere else.
#[derive(Debug)]
pub struct MovedFile {
    pub from: PathBuf,
    pub to: PathBuf,
    pub before: String,
    pub after: String,
}

/// The text of every Kotlin file a plan moves, read before anything changes.
pub struct Snapshot(Vec<(PathBuf, String)>);

impl Snapshot {
    pub fn take(plan: &MovePlan) -> Result<Self> {
        let mut files = Vec::new();
        for step in plan.groups.iter().flat_map(|group| &group.steps) {
            // The temporary name of a move-then-rename does not exist yet; its
            // file is the one the first step already covers.
            if !step.from.exists() {
                continue;
            }
            for entry in WalkDir::new(&step.from) {
                let path = entry?.into_path();
                if path.extension().and_then(|extension| extension.to_str()) != Some("kt") {
                    continue;
                }
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read {}", path.display()))?;
                files.push((path, text));
            }
        }
        Ok(Self(files))
    }

    /// Pair every file with its final place and text once the plan has run.
    pub fn finish(self, plan: &MovePlan) -> Result<Vec<MovedFile>> {
        let mut moved = Vec::new();
        for (from, before) in self.0 {
            let to = plan
                .groups
                .iter()
                .fold(from.clone(), |path, group| relocate(&path, &group.steps));
            let after = std::fs::read_to_string(&to)
                .with_context(|| format!("Cannot read the moved file {}", to.display()))?;
            moved.push(MovedFile {
                from,
                to,
                before,
                after,
            });
        }
        Ok(moved)
    }
}

/// The path the server named, as it exists on disk now. An edit may address a
/// file under its destination before the file has been moved there.
pub fn locate(path: &Path, steps: &[Step]) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }
    for step in steps {
        if let Ok(rest) = path.strip_prefix(&step.to) {
            return join_relative(&step.from, rest);
        }
    }
    path.to_path_buf()
}

/// `Path::join` with an empty path appends a trailing separator, which turns
/// a file into a bogus directory path.
pub fn join_relative(base: &Path, rest: &Path) -> PathBuf {
    if rest.as_os_str().is_empty() {
        base.to_path_buf()
    } else {
        base.join(rest)
    }
}

/// The path of a file after the steps have been carried out.
pub fn relocate(path: &Path, steps: &[Step]) -> PathBuf {
    for step in steps {
        if let Ok(rest) = path.strip_prefix(&step.from) {
            return join_relative(&step.to, rest);
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::kotlin::plan::{Group, Kind};

    fn step(from: &Path, to: &Path) -> Step {
        Step {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            is_dir: false,
        }
    }

    #[test]
    fn a_file_moved_and_then_renamed_maps_from_its_first_place_to_its_last() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            dir.path().join("a/Old.kt"),
            dir.path().join("b/Old.kt"),
            dir.path().join("b/New.kt"),
        );
        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::write(&a, "before").unwrap();
        let plan = MovePlan {
            groups: vec![
                Group {
                    kind: Kind::Move,
                    steps: vec![step(&a, &b)],
                },
                Group {
                    kind: Kind::Rename,
                    steps: vec![step(&b, &c)],
                },
            ],
            notes: Vec::new(),
        };

        let snapshot = Snapshot::take(&plan).unwrap();
        // What the two groups do on disk.
        std::fs::create_dir_all(c.parent().unwrap()).unwrap();
        std::fs::rename(&a, &c).unwrap();
        std::fs::write(&c, "after").unwrap();

        let moved = snapshot.finish(&plan).unwrap();
        assert_eq!(moved.len(), 1);
        assert_eq!((moved[0].from.clone(), moved[0].to.clone()), (a, c));
        assert_eq!(
            (moved[0].before.as_str(), moved[0].after.as_str()),
            ("before", "after")
        );
    }
}
