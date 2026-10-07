//! The command-line arguments of each subcommand: what `clap` parses.

use clap::{Args, ValueHint};
use clap_complete::Shell;

#[derive(Debug, Args)]
pub(super) struct MoveArgs {
    /// Absolute path to the package root (the folder containing tsconfig.json / pyproject.toml / Cargo.toml / settings.gradle.kts etc.). Also settable via REFAC_PROJECT_PATH env var.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// Source file path (relative to project_path or absolute). Repeat for multiple files.
    #[arg(long, required = true, num_args = 1.., value_hint = ValueHint::AnyPath)]
    pub(super) source_path: Vec<String>,

    /// Target file path (relative to project_path or absolute). Must match source count 1:1. Repeat for multiple files.
    #[arg(long, required = true, num_args = 1.., value_hint = ValueHint::AnyPath)]
    pub(super) target_path: Vec<String>,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct MoveModuleArgs {
    /// Cargo package or workspace root. Defaults to the current directory.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// Existing logical module path in one workspace crate, beginning with `crate::`.
    pub(super) source_module: String,

    /// New logical module path in the same crate, beginning with `crate::`.
    pub(super) target_module: String,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct RenameArgs {
    /// Package root containing the authoritative tsconfig.json, the Gradle project root for Kotlin, the folder with go.mod for Go, the Cargo package or workspace root for Rust, the pyright root for Python, or the package folder with pubspec.yaml for Dart. Defaults to the current directory. Also settable via REFAC_PROJECT_PATH env var.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// File containing the symbol (relative to project_path or absolute).
    #[arg(long, value_hint = ValueHint::FilePath)]
    pub(super) file: std::path::PathBuf,

    /// Current name of the symbol, as written in that file.
    #[arg(long)]
    pub(super) symbol: String,

    /// New identifier for the symbol.
    #[arg(long)]
    pub(super) new_name: String,

    /// 1-based line that picks the occurrence when the name refers to several symbols in the file.
    #[arg(long)]
    pub(super) line: Option<u32>,

    /// 1-based byte column on --line, like `rg --column`.
    #[arg(long, requires = "line")]
    pub(super) column: Option<u32>,

    /// Plan and verify the rename, report the edits, and change no files.
    #[arg(long)]
    pub(super) dry_run: bool,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct CompletionsArgs {
    #[arg(value_enum)]
    pub(super) shell: Shell,
}
