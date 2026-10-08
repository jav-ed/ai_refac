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
mod output;
mod rename;

use args::{CompletionsArgs, MoveArgs, MoveModuleArgs, RenameArgs};
use output::{ErrorOutput, MoveModuleSuccessOutput, MoveSuccessOutput};

#[derive(Debug, Parser)]
#[command(
    name = "refac",
    about = "Move or rename source or Markdown files and update all references across the project.",
    long_about = "\
Move or rename source or Markdown files and update all references across the project.

Supported languages: TypeScript, JavaScript, Python, Markdown, Rust, Go, Dart, Kotlin (Android and JVM).
Markdown files, images and other assets, and folders of them can be moved too; after any move, the Markdown links that point at the moved files are updated.
Use `move-module` for semantic Rust module-subtree moves.
Use `rename` to rename a TypeScript/JavaScript, Kotlin, Go, Rust, Python or Dart symbol (variable, parameter, function, type, member) and update every reference.
Renames use a language server (gopls, rust-analyzer, basedpyright, the Dart SDK's server, the Kotlin server). refac starts it for the command and stops it afterwards, so nothing stays in memory. Starting takes seconds (Kotlin about 40), so put several renames of one project into one `rename --batch` call: one start, all or nothing. When a server is missing the error says where refac looked; `refac doctor <language>` shows how to install it. For Kotlin, --project-path is the Gradle project root.
Paths may be absolute or relative to --project-path.

EXAMPLES:
  # Move a single file
  refac move --project-path /my/project \\
    --source-path src/old/name.ts --target-path src/new/name.ts

  # Move multiple files in one call (1:1 mapping)
  refac move --project-path /my/project \\
    --source-path src/a.ts --source-path src/b.ts \\
    --target-path src/x.ts --target-path src/y.ts

  # Move a complete Rust module subtree
  refac move-module --project-path /my/cargo-workspace \\
    crate::engine::matching crate::domain::matching

  # Rename a TypeScript symbol and all of its references
  refac rename --project-path /my/package --file src/lib/util.ts \\
    --symbol total --new-name grandTotal

  # Move a Kotlin file to another package; package line, imports, Android XML follow
  refac move --project-path /my/gradle/project \\
    --source-path app/src/main/kotlin/com/example/ui/Home.kt \\
    --target-path app/src/main/kotlin/com/example/home/Home.kt

  # Rename a Kotlin symbol and all of its references
  refac rename --project-path /my/gradle/project \\
    --file app/src/main/kotlin/com/example/util/Helper.kt \\
    --symbol shout --new-name yell

  # Several renames of one project in one language-server session (JSON list from a file or stdin); all or nothing
  refac rename --project-path /my/crate --batch renames.json
  # renames.json: [{\"file\": \"src/a.rs\", \"symbol\": \"old_a\", \"new_name\": \"new_a\"}, ...]; use --batch - to read it from stdin

  # Rename a Go, Rust, Python or Dart symbol the same way (the file extension picks the language)
  refac rename --project-path /my/module --file shape/shape.go \\
    --symbol Area --new-name Surface

  # A language server is missing: see what refac looked for and how to install it
  refac doctor go
  refac doctor",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Move or rename files and update imports/references. Directories are supported for TypeScript/JavaScript and Kotlin.
    Move(MoveArgs),
    /// Move a complete Rust module subtree and rewrite its semantic references.
    MoveModule(MoveModuleArgs),
    /// Rename a TypeScript/JavaScript, Kotlin, Go, Rust, Python or Dart symbol and update every reference.
    Rename(RenameArgs),
    /// Check the language servers refac starts (gopls, rust-analyzer, ...) and show how to install a missing one.
    Doctor(doctor::DoctorArgs),
    /// Generate shell completions to stdout.
    Completions(CompletionsArgs),
    /// Generate a manpage for the CLI to stdout.
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
    let report =
        crate::drivers::rust::move_module(&project_path, &args.source_module, &args.target_module)
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
            moved_paths: report.moved_paths,
            edited_files: report.edited_files,
        };
        write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })?;
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
