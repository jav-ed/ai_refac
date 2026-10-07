use std::io::{self, Write};
use std::process::ExitCode;

use crate::drivers::symbol_rename::RenameRequest;
use crate::logic::rename::handle_rename;
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

Supported languages: TypeScript, JavaScript, Python, Markdown, Rust, Go, Dart, Kotlin (Android and JVM).
Markdown files, images and other assets, and folders of them can be moved too; after any move, the Markdown links that point at the moved files are updated.
Use `move-module` for semantic Rust module-subtree moves.
Use `rename` to rename a TypeScript/JavaScript or Kotlin symbol (variable, function, class, member) and update every reference.
Kotlin needs the JetBrains Kotlin language server; set REFAC_KOTLIN_SERVER to its install folder. For Kotlin, --project-path is the Gradle project root.
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
    --symbol shout --new-name yell",
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
    /// Rename a TypeScript/JavaScript or Kotlin symbol and update every reference.
    Rename(RenameArgs),
    /// Generate shell completions to stdout.
    Completions(CompletionsArgs),
    /// Generate a manpage for the CLI to stdout.
    Man,
}

#[derive(Debug, Args)]
struct MoveArgs {
    /// Absolute path to the package root (the folder containing tsconfig.json / pyproject.toml / Cargo.toml / settings.gradle.kts etc.). Also settable via REFAC_PROJECT_PATH env var.
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
struct MoveModuleArgs {
    /// Cargo package or workspace root. Defaults to the current directory.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    project_path: Option<std::path::PathBuf>,

    /// Existing logical module path in one workspace crate, beginning with `crate::`.
    source_module: String,

    /// New logical module path in the same crate, beginning with `crate::`.
    target_module: String,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct RenameArgs {
    /// Package root containing the authoritative tsconfig.json, or the Gradle project root for Kotlin. Defaults to the current directory. Also settable via REFAC_PROJECT_PATH env var.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    project_path: Option<std::path::PathBuf>,

    /// File containing the symbol (relative to project_path or absolute).
    #[arg(long, value_hint = ValueHint::FilePath)]
    file: std::path::PathBuf,

    /// Current name of the symbol, as written in that file.
    #[arg(long)]
    symbol: String,

    /// New identifier for the symbol.
    #[arg(long)]
    new_name: String,

    /// 1-based line that picks the occurrence when the name refers to several symbols in the file.
    #[arg(long)]
    line: Option<u32>,

    /// 1-based byte column on --line, like `rg --column`.
    #[arg(long, requires = "line")]
    column: Option<u32>,

    /// Plan and verify the rename, report the edits, and change no files.
    #[arg(long)]
    dry_run: bool,

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
struct MoveModuleSuccessOutput<'a> {
    status: &'static str,
    operation: &'static str,
    project_path: &'a str,
    source_module: &'a str,
    target_module: &'a str,
    moved_paths: usize,
    edited_files: usize,
}

#[derive(Debug, Serialize)]
struct RenameSuccessOutput<'a> {
    status: &'static str,
    operation: &'static str,
    project_path: &'a str,
    file: &'a str,
    symbol: &'a str,
    new_name: &'a str,
    dry_run: bool,
    edits: usize,
    edited_files: usize,
    files: Vec<RenamedFile>,
    notes: &'a [String],
}

#[derive(Debug, Serialize)]
struct RenamedFile {
    path: String,
    edits: usize,
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
        Commands::MoveModule(args) => execute_move_module(args),
        Commands::Rename(args) => execute_rename(args).await,
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

async fn execute_rename(args: RenameArgs) -> Result<(), CliError> {
    let json = args.json;
    let project_path = args
        .project_path
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| CliError {
            json,
            error: error.into(),
        })?;
    let request = RenameRequest {
        project_path: project_path.clone(),
        file: args.file.clone(),
        symbol: args.symbol.clone(),
        new_name: args.new_name.clone(),
        line: args.line,
        column: args.column,
        dry_run: args.dry_run,
    };
    let report = handle_rename(request)
        .await
        .map_err(|error| CliError { json, error })?;

    if json {
        let payload = RenameSuccessOutput {
            status: "ok",
            operation: "rename",
            project_path: &project_path.to_string_lossy(),
            file: &args.file.to_string_lossy(),
            symbol: &args.symbol,
            new_name: &args.new_name,
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
        };
        write_json(io::stdout(), &payload).map_err(|error| CliError { json: true, error })?;
    } else {
        let headline = if report.dry_run {
            "// Dry run: nothing was changed. Planned and verified rename:"
        } else {
            "// Alhamdulillah symbol renamed:"
        };
        println!("{headline}\n{} -> {}", args.symbol, args.new_name);
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
