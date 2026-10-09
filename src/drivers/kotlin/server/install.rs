//! Finding the installed Kotlin language server and the time it may take.

use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

pub const SERVER_ENV: &str = "REFAC_KOTLIN_SERVER";
pub const TIMEOUT_ENV: &str = "REFAC_KOTLIN_TIMEOUT_SECS";
const DEFAULT_TIMEOUT_SECS: u64 = 600;

pub struct Install {
    dir: PathBuf,
    pub build: String,
}

impl Install {
    pub(super) fn executable(&self) -> PathBuf {
        self.dir.join("bin").join("intellij-server")
    }
}

/// The installed server, found like every language server (`crate::servers`):
/// the directory named by `REFAC_KOTLIN_SERVER`, which must hold
/// `bin/intellij-server` and `build.txt`. A missing install is reported with
/// the places looked at and the command that teaches the install.
pub fn locate() -> Result<Install> {
    locate_in(std::env::var_os(SERVER_ENV))
}

pub(super) fn locate_in(configured: Option<OsString>) -> Result<Install> {
    let server = crate::servers::for_language("kotlin")?;
    let project = std::env::current_dir().context("Cannot read the current directory")?;
    let found = crate::servers::locate_with(server, &project, configured)?;
    let Some(build) = found.version.strip_prefix("ILS-") else {
        bail!(
            "{} does not name a Kotlin language server build (expected ILS-<number>, got `{}`). Run `refac doctor kotlin` to see which download refac expects.",
            found
                .folder
                .as_deref()
                .unwrap_or(&found.executable)
                .join("build.txt")
                .display(),
            found.version
        );
    };
    Ok(Install {
        dir: found
            .folder
            .context("The Kotlin server was found without its folder")?,
        build: build.to_string(),
    })
}

pub(super) fn timeout() -> Result<Duration> {
    parse_timeout(std::env::var(TIMEOUT_ENV).ok().as_deref())
}

pub(super) fn parse_timeout(configured: Option<&str>) -> Result<Duration> {
    let Some(value) = configured else {
        return Ok(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    };
    let seconds: u64 = value
        .parse()
        .ok()
        .filter(|seconds| *seconds > 0)
        .with_context(|| {
            format!("{TIMEOUT_ENV} must be a positive number of seconds, got `{value}`")
        })?;
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests;
