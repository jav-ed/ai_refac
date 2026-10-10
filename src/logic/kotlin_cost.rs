//! What a single Kotlin change costs, said to the caller, and the refusal of
//! it, which is on unless the caller turns it off. Every Kotlin command starts the language server and imports
//! the Gradle build first, however few files it changes, so a series of single
//! changes pays that over and over. A command cannot know it is the twentieth:
//! each is a process of its own and nothing stays resident. What it can do is
//! tell the one it sees what the batch form is (`note`), and refuse a single
//! change before anything starts unless the caller adds `--allow-single` or
//! sets `REFAC_KOTLIN_BATCH_ONLY=0` (`refuse_single_move`,
//! `refuse_single_rename`). The refusal is the place where an agent learns what
//! Kotlin costs, so it carries the long explanation (`EXPLANATION`, also
//! `refac guide kotlin`): why it is slow and every option there is.

use super::RefactorRequest;
use super::grouping::route;
use crate::drivers::symbol::rename::RenameRequest;
use anyhow::{Result, bail};
use std::path::Path;
use std::time::Duration;

/// `1`, empty or unset makes refac refuse a single Kotlin change; `0` allows it.
pub const BATCH_ONLY_ENV: &str = "REFAC_KOTLIN_BATCH_ONLY";
/// The argument that lets one single Kotlin change through.
pub const ALLOW_FLAG: &str = "--allow-single";

/// Why a Kotlin change is slow, what it costs and the options, as printed by
/// `refac guide kotlin` and at the end of every refusal.
pub const EXPLANATION: &str = include_str!("kotlin_cost/explanation.txt");

const COST: &str = "Every Kotlin command starts the language server and imports the Gradle build first (about 24 s, 44 s the first time on a machine, and 1.3 to 1.8 GB on a small project), however few files it changes";

fn batch_only() -> Result<bool> {
    parse(std::env::var_os(BATCH_ONLY_ENV).as_deref())
}

/// The value of `REFAC_KOTLIN_BATCH_ONLY`. The refusal is the default, so an
/// unset or empty variable means on and only `0` switches it off. Anything but
/// 1, 0 or empty is an error: a typo must not silently switch the protection
/// off.
fn parse(value: Option<&std::ffi::OsStr>) -> Result<bool> {
    let Some(value) = value else {
        return Ok(true);
    };
    match value.to_str().map(str::trim) {
        Some("") | Some("1") => Ok(true),
        Some("0") => Ok(false),
        _ => bail!(
            "{BATCH_ONLY_ENV} must be 1 (refuse a single Kotlin change, the default) or 0 (allow it), got {value:?}"
        ),
    }
}

/// The pair when `kotlin` (the Kotlin pairs of a request) is one file moved on
/// its own. A folder is already many files in one server start, and several
/// pairs are already a batch.
fn single_file<'a>(
    kotlin: &'a [(String, String)],
    root: Option<&Path>,
) -> Option<&'a (String, String)> {
    match kotlin {
        [pair] if !route::resolve(Path::new(&pair.0), root).is_dir() => Some(pair),
        _ => None,
    }
}

/// Whether the groups of a prepared move (language, pairs) hold exactly one
/// Kotlin file. Asked before the move: a folder that has moved is no longer a
/// folder.
pub(super) fn is_single_move<'a>(
    mut groups: impl Iterator<Item = (&'a str, &'a [(String, String)])>,
    root: Option<&Path>,
) -> bool {
    groups
        .find(|(lang, _)| *lang == "kotlin")
        .is_some_and(|(_, files)| single_file(files, root).is_some())
}

/// Refuses a move of one Kotlin file unless `--allow-single` is given or
/// `REFAC_KOTLIN_BATCH_ONLY=0`. Nothing has been started or changed.
pub fn refuse_single_move(req: &RefactorRequest, allow_single: bool) -> Result<()> {
    check_move(req, allow_single || !batch_only()?)
}

/// `allowed`: the caller said so, or the refusal is switched off.
fn check_move(req: &RefactorRequest, allowed: bool) -> Result<()> {
    if allowed {
        return Ok(());
    }
    let root = req.project_path.as_deref().map(Path::new);
    let targets = req.target_path.as_deref().unwrap_or_default();
    let mut kotlin = Vec::new();
    for (source, target) in req.source_path.iter().zip(targets) {
        if route::language_of(source, root)? == Some("kotlin") {
            kotlin.push((source.clone(), target.clone()));
        }
    }
    let Some((source, target)) = single_file(&kotlin, root) else {
        return Ok(());
    };
    bail!(
        "This is a single Kotlin move, and refac refuses those by default. Nothing was changed.\n{COST}. Run one after the other, each change pays that again.\n\nWhat to do with this request:\n  Several moves: do them all in ONE call, repeating the flags:\n    refac move --source-path {source} --source-path <next.kt> --target-path {target} --target-path <next target>\n  This one move is all there is: run the same command again with {ALLOW_FLAG}.\n  Switch the refusal off for good: {BATCH_ONLY_ENV}=0 (a person's choice; see option 4 below).\n\n{EXPLANATION}"
    )
}

/// Refuses a rename of one Kotlin symbol under the same conditions.
pub fn refuse_single_rename(request: &RenameRequest, allow_single: bool) -> Result<()> {
    // The variable is read for every language, so a typo in it is found at once.
    check_rename(request, allow_single || !batch_only()?)
}

fn check_rename(request: &RenameRequest, allowed: bool) -> Result<()> {
    if allowed || request.file.extension().and_then(|e| e.to_str()) != Some("kt") {
        return Ok(());
    }
    bail!(
        "This is a single Kotlin rename, and refac refuses those by default. Nothing was changed.\n{COST}. Run one after the other, each change pays that again.\n\nWhat to do with this request:\n  Several renames: do them all in ONE call, a JSON list on stdin (or in a file with --batch renames.json):\n    echo '[{{\"file\": \"{}\", \"symbol\": \"{}\", \"new_name\": \"{}\"}}, {{\"file\": \"<next.kt>\", \"symbol\": \"<name>\", \"new_name\": \"<new name>\"}}]' | refac rename --batch -\n  This one rename is all there is: run the same command again with {ALLOW_FLAG}.\n  Switch the refusal off for good: {BATCH_ONLY_ENV}=0 (a person's choice; see option 4 below).\n\n{EXPLANATION}",
        request.file.display(),
        request.symbol,
        request.new_name
    )
}

/// The note a single Kotlin change ends with: what it cost, and the batch form.
/// `what` is `move` or `rename`.
pub fn note(what: &str, elapsed: Duration, dry_run: bool) -> String {
    let dry = if dry_run {
        " This was a dry run: the real run starts the server again."
    } else {
        ""
    };
    format!(
        "This Kotlin {what} took {} s. {COST}; a series of single changes pays that every time. With more than one Kotlin change, do them in one call: moves repeat the flags (`refac move --source-path a.kt --source-path b.kt --target-path pkg/a.kt --target-path pkg/b.kt`), renames take a list (`refac rename --batch renames.json`). refac refuses a single Kotlin change unless you add {ALLOW_FLAG} (or set {BATCH_ONLY_ENV}=0), so add it only when one change is all there is. Why it is slow and every option you have: `refac guide kotlin`.{dry}",
        elapsed.as_secs()
    )
}

#[cfg(test)]
mod tests;
