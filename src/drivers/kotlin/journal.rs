//! Undo log for a refactor that touches many files. Every write and move goes
//! through it, so a failure half way restores the project instead of leaving
//! it half refactored.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

enum Undo {
    Restore { path: PathBuf, original: Vec<u8> },
    MoveBack { from: PathBuf, to: PathBuf },
    RemoveDir(PathBuf),
}

#[derive(Default)]
pub struct Journal {
    undo: Vec<Undo>,
}

impl Journal {
    /// Overwrite an existing file, remembering its bytes.
    pub fn write_file(&mut self, path: &Path, bytes: &[u8]) -> Result<()> {
        let original =
            std::fs::read(path).with_context(|| format!("Cannot read {}", path.display()))?;
        std::fs::write(path, bytes).with_context(|| format!("Cannot write {}", path.display()))?;
        self.undo.push(Undo::Restore {
            path: path.to_path_buf(),
            original,
        });
        Ok(())
    }

    /// Move a file or directory, creating the missing parent directories.
    /// Refuses to replace anything: a move onto an existing path loses data.
    pub fn move_path(&mut self, from: &Path, to: &Path) -> Result<()> {
        if to.exists() {
            bail!("{} already exists", to.display());
        }
        if let Some(parent) = to.parent() {
            self.create_dirs(parent)?;
        }
        std::fs::rename(from, to)
            .with_context(|| format!("Cannot move {} to {}", from.display(), to.display()))?;
        self.undo.push(Undo::MoveBack {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
        });
        Ok(())
    }

    fn create_dirs(&mut self, dir: &Path) -> Result<()> {
        let mut missing: Vec<&Path> = dir.ancestors().take_while(|path| !path.exists()).collect();
        // Outermost first, so a rollback removes the innermost directory first.
        missing.reverse();
        for path in missing {
            std::fs::create_dir(path)
                .with_context(|| format!("Cannot create {}", path.display()))?;
            self.undo.push(Undo::RemoveDir(path.to_path_buf()));
        }
        Ok(())
    }

    /// Undo everything, newest first. Reports every step that failed so the
    /// user knows what to inspect.
    pub fn rollback(self) -> Result<()> {
        let mut failures = Vec::new();
        for step in self.undo.into_iter().rev() {
            let outcome = match &step {
                Undo::Restore { path, original } => std::fs::write(path, original),
                Undo::MoveBack { from, to } => std::fs::rename(to, from),
                Undo::RemoveDir(path) => std::fs::remove_dir(path),
            };
            if let Err(error) = outcome {
                failures.push(format!("{}: {error}", describe(&step)));
            }
        }
        if failures.is_empty() {
            return Ok(());
        }
        bail!(
            "Rollback was incomplete; inspect the working tree:\n{}",
            failures.join("\n")
        )
    }
}

fn describe(step: &Undo) -> String {
    match step {
        Undo::Restore { path, .. } => format!("restoring {}", path.display()),
        Undo::MoveBack { from, to } => {
            format!("moving {} back to {}", to.display(), from.display())
        }
        Undo::RemoveDir(path) => format!("removing {}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_restores_content_moves_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a/File.kt");
        std::fs::create_dir(dir.path().join("a")).unwrap();
        std::fs::write(&file, "old").unwrap();

        let mut journal = Journal::default();
        journal.write_file(&file, b"new").unwrap();
        let target = dir.path().join("b/c/File.kt");
        journal.move_path(&file, &target).unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
        assert!(!file.exists());

        journal.rollback().unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "old");
        assert!(!dir.path().join("b").exists());
    }

    #[test]
    fn a_move_never_replaces_an_existing_path() {
        let dir = tempfile::tempdir().unwrap();
        let (from, to) = (dir.path().join("From.kt"), dir.path().join("To.kt"));
        std::fs::write(&from, "from").unwrap();
        std::fs::write(&to, "to").unwrap();

        let error = Journal::default().move_path(&from, &to).unwrap_err();
        assert!(error.to_string().contains("already exists"), "{error}");
        assert_eq!(std::fs::read_to_string(&to).unwrap(), "to");
    }

    #[test]
    fn rollback_reports_a_step_it_could_not_undo() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("File.kt");
        std::fs::write(&file, "old").unwrap();
        let mut journal = Journal::default();
        let target = dir.path().join("sub/File.kt");
        journal.move_path(&file, &target).unwrap();
        // Something else now sits where the file has to go back to.
        std::fs::create_dir(&file).unwrap();

        let error = journal.rollback().unwrap_err().to_string();
        assert!(error.contains("incomplete"), "{error}");
        assert!(error.contains("File.kt"), "{error}");
    }
}
