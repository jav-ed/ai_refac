use std::io::{self, Write};
use std::process::ExitCode;

use crate::logic::{RefactorRequest, handle_refactor};
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use clap_mangen::Man;
use serde::Serialize;

mod args;
mod doctor;
mod guide;
mod help;
mod output;
mod rename;

use args::{CompletionsArgs, MoveArgs, MoveModuleArgs, RenameArgs};
use output::{ErrorOutput, MoveModuleSuccessOutput, MoveSuccessOutput, MovedPath, RenamedFile};

#[derive(Debug, Parser)]
#[command(
    name = "refac",
    about = help::top::ABOUT,
    long_about = help::top::LONG_ABOUT,
    after_long_help = help::top::AFTER_LONG_HELP,
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        about = help::move_files::ABOUT,
        long_about = help::move_files::LONG_ABOUT,
        after_long_help = help::move_files::AFTER_LONG_HELP
    )]
    Move(MoveArgs),
    #[command(
        about = help::move_module::ABOUT,
        long_about = help::move_module::LONG_ABOUT,
        after_long_help = help::move_module::AFTER_LONG_HELP
    )]
    MoveModule(MoveModuleArgs),
    #[command(
        about = help::rename::ABOUT,
        long_about = help::rename::LONG_ABOUT,
        after_long_help = help::rename::AFTER_LONG_HELP
    )]
    Rename(RenameArgs),
    #[command(
        about = help::doctor::ABOUT,
        long_about = help::doctor::LONG_ABOUT,
        after_long_help = help::doctor::AFTER_LONG_HELP
    )]
    Doctor(doctor::DoctorArgs),
    /// In-depth documentation by topic: languages, safety, batching, output, servers.
    ///
    /// `refac guide` lists the topics; `refac guide <topic>` prints one; `refac guide all` prints
    /// every topic. The texts are inside the binary, so they are there when the repository is not.
    Guide(guide::GuideArgs),
    /// Print shell completions (bash, zsh, fish, elvish, powershell) to stdout.
    ///
    /// Example: `refac completions bash > ~/.local/share/bash-completion/completions/refac`
    Completions(CompletionsArgs),
    /// Print the manual page (roff) to stdout.
    ///
    /// Example: `refac man | man -l -`
    Man,
}

#[derive(Debug)]
struct CliError {
    json: bool,
    error: anyhow::Error,
}

pub async fn run() -> ExitCode {
    let cli = Cli::parse();

    match execute(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            render_error(&err);
            ExitCode::FAILURE
        }
    }
}

async fn execute(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Move(args) => execute_move(args).await,
        Commands::MoveModule(args) => execute_move_module(args),
        Commands::Rename(args) => rename::execute_rename(args).await,
        Commands::Doctor(args) => doctor::execute_doctor(args).await,
        Commands::Guide(args) => guide::execute_guide(args),
        Commands::Completions(args) => execute_completions(args),
        Commands::Man => execute_man(),
    }
}

async fn execute_move(args: MoveArgs) -> Result<(), CliError> {
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

fn execute_move_module(args: MoveModuleArgs) -> Result<(), CliError> {
    let project_path = args
        .project_path
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| CliError {
            json: args.json,
            error: error.into(),
        })?;
    let report = crate::drivers::rust::move_module(
        &project_path,
        &args.source_module,
        &args.target_module,
        args.dry_run,
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
        println!(
            "// The Cargo check that proves the result still compiles runs only on a real move."
        );
        println!("// Run the same command without --dry-run to apply the move.");
    } else {
        println!(
            "// Alhamdulillah Rust module moved semantically:\n{} -> {}\n// {} filesystem path(s) moved; {} source file(s) updated.",
            args.source_module, args.target_module, report.moved_paths, report.edited_files
        );
    }

    Ok(())
}

fn execute_completions(args: CompletionsArgs) -> Result<(), CliError> {
    let mut command = Cli::command();
    let command_name = command.get_name().to_string();
    generate(args.shell, &mut command, command_name, &mut io::stdout());
    Ok(())
}

fn execute_man() -> Result<(), CliError> {
    let command = Cli::command();
    let mut buffer = Vec::new();
    Man::new(command)
        .render(&mut buffer)
        .map_err(|error| CliError {
            json: false,
            error: error.into(),
        })?;

    io::stdout().write_all(&buffer).map_err(|error| CliError {
        json: false,
        error: error.into(),
    })?;

    Ok(())
}

fn render_error(err: &CliError) {
    if err.json {
        let payload = ErrorOutput {
            status: "error",
            error: &format!("{:#}", err.error),
        };
        let _ = write_json(io::stderr(), &payload);
    } else {
        eprintln!("{:#}", err.error);
    }
}

fn write_json<W: Write, T: Serialize>(mut writer: W, value: &T) -> Result<()> {
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
