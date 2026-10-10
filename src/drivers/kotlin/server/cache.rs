//! The server's system directory: the caches and indexes it writes while it
//! imports a project (about 120 MB). A start on an empty one spends about 20
//! of its 32 seconds indexing the JDK and the Kotlin libraries, and a start on
//! one that an earlier run left is ready after 9 seconds, whatever the project
//! and its path (measured on 2026-10-10; edits, new files, deletions and moves
//! made while no server ran are seen, the server compares the disk with its
//! index at start).
//!
//! So a run does not delete what it learned. Every run works in a private copy
//! of the warm directory, never in the warm directory itself: the server locks
//! the directory it runs in, and two commands (or the several servers of a
//! test program) must not wait for each other. A run that ends cleanly
//! publishes its directory as the new warm one, so the next run starts from
//! the libraries of every project that ran before it. A run that failed
//! publishes nothing.
//!
//! The warm directory is per server build (the index format changes with it),
//! lives in `$REFAC_KOTLIN_CACHE`, else `$XDG_CACHE_HOME/refac`, else
//! `~/.cache/refac`, and is deleted when it grows past 2 GB. `off` in
//! `REFAC_KOTLIN_CACHE` gives every run an empty directory, as before.

use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

pub const CACHE_ENV: &str = "REFAC_KOTLIN_CACHE";
const OFF: &str = "off";
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// The warm directory of one server build and the lock that guards it.
pub struct Warm {
    root: PathBuf,
}

/// The directory one server run works in.
pub struct SystemDir {
    work: TempDir,
    warm: Option<Warm>,
}

/// Where the warm directory of `build` lives, from the settings of the
/// environment. `None` when the cache is switched off or no place is known.
pub fn locate(
    build: &str,
    configured: Option<&str>,
    xdg_cache: Option<OsString>,
    home: Option<OsString>,
) -> Result<Option<Warm>> {
    let base = match configured {
        Some(OFF) => return Ok(None),
        Some("") => bail!("{CACHE_ENV} must be `{OFF}` or a directory, got an empty value"),
        Some(directory) => PathBuf::from(directory),
        None => match (xdg_cache.filter(|dir| !dir.is_empty()), home) {
            (Some(xdg), _) => PathBuf::from(xdg).join("refac"),
            (None, Some(home)) => PathBuf::from(home).join(".cache").join("refac"),
            (None, None) => return Ok(None),
        },
    };
    Ok(Some(Warm::at(base.join(format!("kotlin-{build}")))))
}

impl Warm {
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    fn warm(&self) -> PathBuf {
        self.root.join("warm")
    }

    /// The lock file is never removed, so every process locks the same file.
    fn lock(&self, exclusive: bool) -> Result<File> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("Cannot create the cache {}", self.root.display()))?;
        let file = File::create(self.root.join(".lock"))?;
        if exclusive {
            file.lock()?;
        } else {
            file.lock_shared()?;
        }
        Ok(file)
    }

    /// Copies the warm directory, if there is one, into `into`.
    fn seed(&self, into: &Path) -> Result<()> {
        let _lock = self.lock(false)?;
        if !self.warm().is_dir() {
            return Ok(());
        }
        copy_tree(&self.warm(), into).with_context(|| {
            format!(
                "The warm cache {} could not be copied. Delete it, or set {CACHE_ENV}={OFF}",
                self.warm().display()
            )
        })
    }

    /// Makes `from` the warm directory, or deletes the warm directory when
    /// `from` is bigger than `max_bytes`.
    fn publish(&self, from: &Path, max_bytes: u64) -> Result<()> {
        let _lock = self.lock(true)?;
        let (next, old) = (self.root.join("warm.next"), self.root.join("warm.old"));
        remove(&next)?;
        remove(&old)?;
        let size = size_of(from)?;
        if size > max_bytes {
            remove(&self.warm())?;
            bail!(
                "The cache of the Kotlin server grew to {} MB, over the limit of {} MB, and was deleted",
                size >> 20,
                max_bytes >> 20
            );
        }
        copy_tree(from, &next)?;
        if self.warm().exists() {
            fs::rename(self.warm(), &old)?;
        }
        fs::rename(&next, self.warm())?;
        remove(&old)
    }
}

impl SystemDir {
    /// A directory for a run of the server `build`, holding what the earlier
    /// runs left.
    pub fn open(build: &str) -> Result<Self> {
        let warm = locate(
            build,
            std::env::var(CACHE_ENV).ok().as_deref(),
            std::env::var_os("XDG_CACHE_HOME"),
            std::env::var_os("HOME"),
        )?;
        Self::with(warm)
    }

    pub fn with(warm: Option<Warm>) -> Result<Self> {
        let work = tempfile::Builder::new().prefix("refac-kotlin-").tempdir()?;
        if let Some(warm) = &warm {
            warm.seed(work.path())?;
        }
        Ok(Self { work, warm })
    }

    pub fn path(&self) -> &Path {
        self.work.path()
    }

    /// The run ended cleanly and the server has exited, so the directory is
    /// complete: it becomes the warm one. The private directory is deleted.
    pub fn keep(self) -> Result<()> {
        match &self.warm {
            Some(warm) => warm.publish(self.work.path(), MAX_BYTES),
            None => Ok(()),
        }
    }

    #[cfg(test)]
    fn keep_with_limit(self, max_bytes: u64) -> Result<()> {
        self.warm
            .as_ref()
            .expect("a warm cache")
            .publish(self.work.path(), max_bytes)
    }
}

fn remove(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("Cannot delete {}", path.display()))?;
    }
    Ok(())
}

fn size_of(dir: &Path) -> Result<u64> {
    let mut bytes = 0;
    for entry in WalkDir::new(dir) {
        bytes += entry?.metadata()?.len();
    }
    Ok(bytes)
}

/// Files and folders only: a link in a cache would point out of it.
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in WalkDir::new(from) {
        let entry = entry?;
        let target = to.join(entry.path().strip_prefix(from)?);
        let kind = entry.file_type();
        if kind.is_dir() {
            fs::create_dir_all(&target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)?;
        } else {
            bail!("{} is neither a file nor a folder", entry.path().display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
