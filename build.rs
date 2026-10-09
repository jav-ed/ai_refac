//! Stamps the release binary with the commit it was built from, so
//! `refac --version` shows whether the installed command is current
//! (`scripts/install.sh` builds, links and checks it). Only the release profile
//! is stamped: a debug or test build must not recompile after every commit.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// What the installed binary is built from: code, helper scripts, lock files.
const SOURCES: [&str; 5] = ["src", "scripts", "Cargo.toml", "Cargo.lock", "build.rs"];

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// `2026-10-09 22:10 UTC` from seconds since the epoch (days to civil date).
fn utc(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let minutes = seconds % 86_400 / 60;
    format!(
        "{year}-{month:02}-{day:02} {:02}:{:02} UTC",
        minutes / 60,
        minutes % 60
    )
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("PROFILE").as_deref() != Ok("release") {
        println!("cargo:rustc-env=REFAC_BUILD=dev build");
        return;
    }
    for path in [".git/HEAD"].iter().chain(SOURCES.iter()) {
        println!("cargo:rerun-if-changed={path}");
    }
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
        println!("cargo:rerun-if-changed=.git/{reference}");
    }

    let built = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "unknown time".to_string(), |d| utc(d.as_secs()));
    let stamp = match git(&["rev-parse", "--short", "HEAD"]) {
        Some(commit) => {
            let mut status = vec!["status", "--porcelain", "--"];
            status.extend(SOURCES);
            let dirty = git(&status).is_some_and(|changes| !changes.is_empty());
            format!(
                "{commit}{}, built {built}",
                if dirty { "+dirty" } else { "" }
            )
        }
        None => format!("no git checkout, built {built}"),
    };
    println!("cargo:rustc-env=REFAC_BUILD={stamp}");
}
