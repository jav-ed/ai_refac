//! `move-module --dry-run --check`: the plan of a move that includes the Cargo
//! check that follows a real one. A compile needs the files in their new places,
//! so the real move runs on a throw-away copy of the workspace and the project
//! itself is not touched. The copy has a build folder of its own, so the check
//! takes as long as a first build of the workspace.

use super::planner::{MoveMode, MoveModuleReport, move_module};
use crate::drivers::preview::copy::{CopyPlan, ProjectRoot};
use anyhow::{Result, anyhow};
use std::path::Path;

pub(super) fn plan_on_copy(
    root: &Path,
    source_path: &str,
    target_path: &str,
) -> Result<MoveModuleReport> {
    let project = ProjectRoot::at(Some(root))?;
    let copy = project.copy(&CopyPlan {
        // Build output; the copy compiles into a folder of its own.
        skip: &["target"],
        ..CopyPlan::default()
    })?;
    let report = move_module(copy.path(), source_path, target_path, MoveMode::Apply).map_err(
        |error| {
            anyhow!(
                "{}\n(This was the move carried out on a copy of the workspace, with its own build folder; the project itself was not touched. The copy holds the workspace folder only, so a path dependency outside it cannot be found there.)",
                copy.about_the_project(&format!("{error:#}"))
            )
        },
    )?;
    // The paths of the report are relative to the workspace, so they read the
    // same for the copy and the project.
    Ok(MoveModuleReport {
        dry_run: true,
        ..report
    })
}
