//! Cargo writes a missing `Cargo.lock` while the workspace loads. A dry run
//! promises to change no file, so it removes the lock files its own load
//! created (and only those) when it ends, also when it ends in an error.

use std::path::{Path, PathBuf};

pub struct NewLockfiles {
    /// The places a `Cargo.lock` would be written that have none yet: the
    /// project folder and every folder above it that holds a manifest, since
    /// the lock lives next to the manifest of the workspace root.
    missing: Vec<PathBuf>,
}

impl NewLockfiles {
    pub fn watch(root: &Path) -> Self {
        let missing = root
            .ancestors()
            .filter(|folder| folder.join("Cargo.toml").exists())
            .map(|folder| folder.join("Cargo.lock"))
            .filter(|lock| !lock.exists())
            .collect();
        Self { missing }
    }
}

impl Drop for NewLockfiles {
    fn drop(&mut self) {
        for lock in &self.missing {
            // Not being able to remove it only leaves the lock cargo wrote.
            if lock.exists() {
                let _ = std::fs::remove_file(lock);
            }
        }
    }
}

#[cfg(test)]
mod tests;
