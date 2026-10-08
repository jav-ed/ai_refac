//! `refac rename`: one symbol, or a batch of them in one language-server
//! session.

use super::args::RenameArgs;
use super::output::{RenameBatchOutput, RenameResult, RenameSuccessOutput, RenamedFile};
use super::{CliError, write_json};
use crate::drivers::symbol::rename::{RenameReport, RenameRequest};
use crate::logic::rename::{handle_rename, handle_rename_batch};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// One entry of a batch file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchEntry {
    file: PathBuf,
    symbol: String,
    new_name: String,
    line: Option<u32>,
    column: Option<u32>,
}

pub(super) async fn execute_rename(args: RenameArgs) -> Result<(), CliError> {
    let json = args.json;
    let fail = |error: anyhow::Error| CliError { json, error };
    let project_path = args
        .project_path
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| fail(error.into()))?;
    match args.batch {
        Some(ref batch) => {
            let requests = read_batch(batch, &project_path, args.dry_run).map_err(fail)?;
            let reports = handle_rename_batch(requests.clone()).await.map_err(fail)?;
            print_batch(&project_path, &requests, &reports, args.dry_run, json)
        }
        None => {
            // Clap requires these three unless a batch is given.
            let (Some(file), Some(symbol), Some(new_name)) =
                (args.file, args.symbol, args.new_name)
            else {
                return Err(fail(anyhow::anyhow!(
                    "rename needs --file, --symbol and --new-name (or --batch)"
                )));
            };
            let request = RenameRequest {
                project_path: project_path.clone(),
                file,
                symbol,
                new_name,
                line: args.line,
                column: args.column,
                dry_run: args.dry_run,
            };
            let report = handle_rename(request.clone()).await.map_err(fail)?;
            print_single(&project_path, &request, &report, json)
        }
    }
}

/// The requests of a batch file, or of stdin for `-`.
fn read_batch(source: &Path, project_path: &Path, dry_run: bool) -> Result<Vec<RenameRequest>> {
    let text = if source == Path::new("-") {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .context("Cannot read the batch from stdin")?;
        text
    } else {
        std::fs::read_to_string(source)
            .with_context(|| format!("Cannot read the batch file {}", source.display()))?
    };
    let entries: Vec<BatchEntry> = serde_json::from_str(&text).with_context(|| {
        format!(
            "{} is not a batch: expected a JSON list of {{\"file\", \"symbol\", \"new_name\"}} objects, each with an optional \"line\" and \"column\"",
            source.display()
        )
    })?;
    Ok(entries
        .into_iter()
        .map(|entry| RenameRequest {
            project_path: project_path.to_path_buf(),
            file: entry.file,
            symbol: entry.symbol,
            new_name: entry.new_name,
            line: entry.line,
            column: entry.column,
            dry_run,
        })
        .collect())
}

fn result<'a>(request: &'a RenameRequest, report: &'a RenameReport) -> RenameResult<'a> {
    RenameResult {
        file: request.file.to_str().unwrap_or_default(),
        symbol: &request.symbol,
        new_name: &request.new_name,
        dry_run: report.dry_run,
        edits: report.edits,
        edited_files: report.files.len(),
        files: report
            .files
            .iter()
            .map(|(path, edits)| RenamedFile {
                path: path.to_string_lossy().into_owned(),
                edits: *edits,
            })
            .collect(),
        notes: &report.notes,
    }
}

fn print_single(
    project_path: &Path,
    request: &RenameRequest,
    report: &RenameReport,
    json: bool,
) -> Result<(), CliError> {
    if json {
        let payload = RenameSuccessOutput {
            status: "ok",
            operation: "rename",
            project_path: &project_path.to_string_lossy(),
            rename: result(request, report),
        };
        write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })?;
        return Ok(());
    }
    let headline = if report.dry_run {
        "// Dry run: nothing was changed. Planned and verified rename:"
    } else {
        "// Alhamdulillah symbol renamed:"
    };
    println!("{headline}");
    print_rename(request, report);
    if report.dry_run {
        println!("// Run the same command without --dry-run to write these edits.");
    }
    Ok(())
}

fn print_batch(
    project_path: &Path,
    requests: &[RenameRequest],
    reports: &[RenameReport],
    dry_run: bool,
    json: bool,
) -> Result<(), CliError> {
    let edits: usize = reports.iter().map(|report| report.edits).sum();
    let files: BTreeSet<&PathBuf> = reports
        .iter()
        .flat_map(|report| report.files.iter().map(|(path, _)| path))
        .collect();
    if json {
        let payload = RenameBatchOutput {
            status: "ok",
            operation: "rename-batch",
            project_path: &project_path.to_string_lossy(),
            dry_run,
            edits,
            edited_files: files.len(),
            renames: requests
                .iter()
                .zip(reports)
                .map(|(q, r)| result(q, r))
                .collect(),
        };
        write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })?;
        return Ok(());
    }
    let headline = if dry_run {
        "// Dry run: nothing was changed. Planned and verified"
    } else {
        "// Alhamdulillah"
    };
    let verb = if dry_run { "" } else { " renamed" };
    println!(
        "{headline} {} symbols{verb} in one language-server session:",
        requests.len()
    );
    for (request, report) in requests.iter().zip(reports) {
        print_rename(request, report);
    }
    println!("// {edits} edit(s) in {} file(s) in total.", files.len());
    if dry_run {
        println!("// Run the same command without --dry-run to write these edits.");
    }
    Ok(())
}

/// One rename: the names, the files with their edits, the notes.
fn print_rename(request: &RenameRequest, report: &RenameReport) {
    println!("{} -> {}", request.symbol, request.new_name);
    for (path, edits) in &report.files {
        println!(
            "// {} ({edits} edit{})",
            path.display(),
            if *edits == 1 { "" } else { "s" }
        );
    }
    println!(
        "// {} edit(s) in {} file(s).",
        report.edits,
        report.files.len()
    );
    for note in &report.notes {
        println!("// Note: {note}");
    }
}
