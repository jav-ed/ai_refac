//! The move commands: `move` (and `move --dry-run`) and `move-module`.

use super::args::{MoveArgs, MoveModuleArgs};
use super::output::{
    MoveDryRunOutput, MoveModuleSuccessOutput, MoveSuccessOutput, MovedPath, RenamedFile,
};
use super::{CliError, write_json};
use crate::drivers::rust::MoveMode;
use crate::logic::{RefactorRequest, handle_refactor, plan_refactor};
use anyhow::Result;
use std::io;

pub(super) async fn execute_move(args: MoveArgs) -> Result<(), CliError> {
    if args.source_path.len() != args.target_path.len() {
        return Err(CliError {
            json: args.json,
            error: anyhow::anyhow!(
                "Mismatch check: Source count ({}) != Target count ({})",
                args.source_path.len(),
                args.target_path.len()
            ),
        });
    }

    let req = RefactorRequest {
        source_path: args.source_path.clone(),
        target_path: Some(args.target_path.clone()),
        operation: "move".to_string(),
        project_path: args
            .project_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
    };

    crate::logic::kotlin_cost::refuse_single_move(&req, args.allow_single).map_err(|error| {
        CliError {
            json: args.json,
            error,
        }
    })?;

    if args.dry_run {
        return execute_move_dry_run(&args, req).await;
    }

    match handle_refactor(req).await {
        Ok(result) => {
            if args.json {
                let payload = MoveSuccessOutput {
                    status: "ok",
                    operation: "move",
                    project_path: args
                        .project_path
                        .as_deref()
                        .and_then(std::path::Path::to_str),
                    source_path: &args.source_path,
                    target_path: &args.target_path,
                    result: &result,
                };
                write_json(io::stdout(), &payload)
                    .map_err(|error| CliError { json: true, error })?;
            } else {
                println!("{result}");
            }

            Ok(())
        }
        Err(error) => Err(CliError {
            json: args.json,
            error,
        }),
    }
}

async fn execute_move_dry_run(args: &MoveArgs, req: RefactorRequest) -> Result<(), CliError> {
    let plan = plan_refactor(req).await.map_err(|error| CliError {
        json: args.json,
        error,
    })?;
    if !args.json {
        println!("{}", plan.text);
        return Ok(());
    }
    let payload = MoveDryRunOutput {
        status: "ok",
        operation: "move",
        dry_run: true,
        project_path: args
            .project_path
            .as_deref()
            .and_then(std::path::Path::to_str),
        source_path: &args.source_path,
        target_path: &args.target_path,
        moved_paths: plan.moves.len(),
        edited_files: plan.files.len(),
        edits: plan.files.iter().map(|(_, edits)| edits).sum(),
        files: plan
            .files
            .iter()
            .map(|(path, edits)| RenamedFile {
                path: path.clone(),
                edits: *edits,
            })
            .collect(),
        moves: plan
            .moves
            .iter()
            .map(|(from, to)| MovedPath {
                from: from.clone(),
                to: to.clone(),
            })
            .collect(),
        notes: &plan.notes,
        result: &plan.text,
    };
    write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })
}

pub(super) fn execute_move_module(args: MoveModuleArgs) -> Result<(), CliError> {
    let project_path = args
        .project_path
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| CliError {
            json: args.json,
            error: error.into(),
        })?;
    let mode = match (args.dry_run, args.check) {
        (false, _) => MoveMode::Apply,
        (true, false) => MoveMode::Plan,
        (true, true) => MoveMode::PlanAndCompile,
    };
    let report = crate::drivers::rust::move_module(
        &project_path,
        &args.source_module,
        &args.target_module,
        mode,
    )
    .map_err(|error| CliError {
        json: args.json,
        error,
    })?;

    if args.json {
        let project_display = project_path.to_string_lossy();
        let payload = MoveModuleSuccessOutput {
            status: "ok",
            operation: "move-module",
            project_path: &project_display,
            source_module: &args.source_module,
            target_module: &args.target_module,
            dry_run: report.dry_run,
            compiled: report.compiled,
            moved_paths: report.moved_paths,
            edited_files: report.edited_files,
            edits: report.edits,
            files: report
                .files
                .iter()
                .map(|(path, edits)| RenamedFile {
                    path: path.to_string_lossy().into_owned(),
                    edits: *edits,
                })
                .collect(),
            moves: report
                .moves
                .iter()
                .map(|(from, to)| MovedPath {
                    from: from.to_string_lossy().into_owned(),
                    to: to.to_string_lossy().into_owned(),
                })
                .collect(),
        };
        write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })?;
        return Ok(());
    }

    if report.dry_run {
        println!("// Dry run: nothing was changed. Planned Rust module move:");
        println!("{} -> {}", args.source_module, args.target_module);
        for (from, to) in &report.moves {
            println!("// move {} -> {}", from.display(), to.display());
        }
        for (path, edits) in &report.files {
            println!(
                "// {} ({edits} edit{})",
                path.display(),
                if *edits == 1 { "" } else { "s" }
            );
        }
        println!(
            "// {} edit(s) in {} file(s); {} path(s) would move.",
            report.edits,
            report.files.len(),
            report.moved_paths
        );
        if report.compiled {
            println!(
                "// The moved workspace compiled (cargo check --workspace --all-targets, on a copy of the workspace)."
            );
        } else {
            println!(
                "// The Cargo check that proves the result still compiles was not made: add --check to make it on a copy of the workspace (as slow as a first build), or it runs on the real move."
            );
        }
        println!("// Run the same command without --dry-run to apply the move.");
    } else {
        println!(
            "// Alhamdulillah Rust module moved semantically:\n{} -> {}\n// {} filesystem path(s) moved; {} source file(s) updated.",
            args.source_module, args.target_module, report.moved_paths, report.edited_files
        );
    }

    Ok(())
}
