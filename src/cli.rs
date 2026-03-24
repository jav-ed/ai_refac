use std::io::{self, Write};
use std::process::ExitCode;

use crate::logic::{RefactorRequest, handle_refactor};
use anyhow::Result;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{Shell, generate};
use clap_mangen::Man;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "refac",
    about = "Move or rename source or Markdown files and update all references across the project.",
    long_about = "\
Move or rename source or Markdown files and update all references across the project.

Supported languages: TypeScript, JavaScript, Python, Markdown, Rust, Go, Dart.
Only individual FILES are supported — directory moves are not.
Paths may be absolute or relative to --project-path.

EXAMPLES:
  # Move a single file
  refac move --project-path /my/project \\
    --source-path src/old/name.ts --target-path src/new/name.ts

  # Move multiple files in one call (1:1 mapping)
  refac move --project-path /my/project \\
    --source-path src/a.ts --source-path src/b.ts \\
    --target-path src/x.ts --target-path src/y.ts",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Move or rename files and update imports/references. Only files are supported, not directories.
    Move(MoveArgs),
    /// Generate shell completions to stdout.
    Completions(CompletionsArgs),
    /// Generate a manpage for the CLI to stdout.
    Man,
}

#[derive(Debug, Args)]
struct MoveArgs {
    /// Absolute path to the package root (the folder containing tsconfig.json / pyproject.toml / Cargo.toml etc.). Also settable via REFAC_PROJECT_PATH env var.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    project_path: Option<std::path::PathBuf>,

    /// Source file path (relative to project_path or absolute). Repeat for multiple files.
    #[arg(long, required = true, num_args = 1.., value_hint = ValueHint::AnyPath)]
    source_path: Vec<String>,

    /// Target file path (relative to project_path or absolute). Must match source count 1:1. Repeat for multiple files.
    #[arg(long, required = true, num_args = 1.., value_hint = ValueHint::AnyPath)]
    target_path: Vec<String>,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct CompletionsArgs {
    #[arg(value_enum)]
    shell: Shell,
}

#[derive(Debug)]
struct CliError {
    json: bool,
    error: anyhow::Error,
}

#[derive(Debug, Serialize)]
struct MoveSuccessOutput<'a> {
    status: &'static str,
    operation: &'static str,
    project_path: Option<&'a str>,
    source_path: &'a [String],
    target_path: &'a [String],
    result: &'a str,
}

#[derive(Debug, Serialize)]
struct ErrorOutput<'a> {
    status: &'static str,
    error: &'a str,
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
