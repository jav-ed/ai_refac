//! Finding the installed Kotlin language server and the time it may take.

use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

pub const SERVER_ENV: &str = "REFAC_KOTLIN_SERVER";
pub const TIMEOUT_ENV: &str = "REFAC_KOTLIN_TIMEOUT_SECS";
const DEFAULT_TIMEOUT_SECS: u64 = 600;
pub const GRADLE_IDLE_ENV: &str = "REFAC_KOTLIN_GRADLE_IDLE_SECS";
/// Gradle keeps the daemon of the server's import for three hours by default
/// (about 0.5 GB); nothing refac starts may outlive the command, so the
/// default is a few seconds.
const DEFAULT_GRADLE_IDLE_SECS: u64 = 10;
const MAX_GRADLE_IDLE_SECS: u64 = 3600;

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

/// How long, in milliseconds, the Gradle daemon of the server's import may sit
/// idle after the command before it stops itself (`REFAC_KOTLIN_GRADLE_IDLE_SECS`).
pub(super) fn gradle_idle_ms() -> Result<u64> {
    parse_gradle_idle(std::env::var(GRADLE_IDLE_ENV).ok().as_deref())
}

/// Unset, empty or `0` is the default of ten seconds: keeping the daemon is
/// something the caller asks for. Anything but a number of seconds up to an
/// hour is an error, so a typo cannot leave a daemon behind for hours.
pub(super) fn parse_gradle_idle(configured: Option<&str>) -> Result<u64> {
    let seconds = match configured.map(str::trim) {
        None | Some("") | Some("0") => DEFAULT_GRADLE_IDLE_SECS,
        Some(value) => value
            .parse()
            .ok()
            .filter(|seconds| (1..=MAX_GRADLE_IDLE_SECS).contains(seconds))
            .with_context(|| {
                format!(
                    "{GRADLE_IDLE_ENV} must be a number of seconds from 1 to {MAX_GRADLE_IDLE_SECS} (0 or unset: {DEFAULT_GRADLE_IDLE_SECS}), got `{value}`"
                )
            })?,
    };
    Ok(seconds * 1000)
}

#[cfg(test)]
mod tests;
