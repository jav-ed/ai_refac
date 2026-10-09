//! The command-line arguments of each subcommand: what `clap` parses.
//!
//! Every flag has a first line that stands alone (`-h` shows only that) and,
//! where it helps, a paragraph below it that `--help` adds.

use clap::{Args, ValueHint};
use clap_complete::Shell;

#[derive(Debug, Args)]
pub(super) struct MoveArgs {
    /// Package root of the language (the folder with tsconfig.json, go.mod, Cargo.toml, ...).
    ///
    /// TypeScript/JavaScript: the folder with the tsconfig.json that includes every caller.
    /// Kotlin: the Gradle root (settings.gradle.kts). Go: go.mod. Rust: Cargo.toml.
    /// Dart: pubspec.yaml. Python and Markdown: the project folder. Not the monorepo root.
    /// Without it, relative paths are taken from the current directory.
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// File or folder to move (absolute, or relative to --project-path). Repeat for several.
    ///
    /// The n-th --source-path goes to the n-th --target-path, so both flags need the same count.
    #[arg(long, required = true, num_args = 1.., value_name = "PATH", value_hint = ValueHint::AnyPath)]
    pub(super) source_path: Vec<String>,

    /// Where it goes (absolute, or relative to --project-path). Repeat for several.
    ///
    /// A folder in the path that does not exist yet is created.
    #[arg(long, required = true, num_args = 1.., value_name = "PATH", value_hint = ValueHint::AnyPath)]
    pub(super) target_path: Vec<String>,

    /// Plan the move, print what it would do, and change no file.
    ///
    /// Lists, per language, the paths that would move (also the files that move with a Go package)
    /// and the edits per file, and refuses what the real move refuses. Language servers start
    /// as for a real move and stop afterwards. Python (Rope) and Kotlin are planned by carrying the
    /// move out on a throw-away copy of the project, so their own checks run too (limit 500 MiB,
    /// REFAC_DRY_RUN_COPY_MAX_MB; Rope copies only the .py and .pyi files). TypeScript resolves every
    /// rewritten import against the files as they will be after the move, as the real move does.
    #[arg(long)]
    pub(super) dry_run: bool,

    /// Kotlin only: let a single Kotlin change through when REFAC_KOTLIN_BATCH_ONLY=1 refuses it.
    ///
    /// Every Kotlin command starts the language server and imports the Gradle build (30 to 40 s,
    /// 1.3 to 1.8 GB), however few files it changes, so a series of single changes is slow. With
    /// REFAC_KOTLIN_BATCH_ONLY=1 a single Kotlin move is refused with the command that does several
    /// in one call; this argument says that the one change is all there is. It has no effect on
    /// other languages, on a batch, or when the variable is not set.
    #[arg(long)]
    pub(super) allow_single: bool,

    /// Print one JSON document instead of text (an error is JSON on stderr).
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct MoveModuleArgs {
    /// Cargo package or workspace root (default: the current directory).
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// The module as it is now, e.g. `crate::engine::matching`.
    ///
    /// Must start with `crate::` and name a module of one crate of the workspace.
    pub(super) source_module: String,

    /// The module as it should be, e.g. `crate::domain::matching`.
    ///
    /// Must start with `crate::` and lie in the same crate as the source.
    pub(super) target_module: String,

    /// Plan the move, print what it would do, and change no file.
    ///
    /// The plan is checked for conflicts. The Cargo check that proves the moved workspace still
    /// compiles is made only with --check.
    #[arg(long)]
    pub(super) dry_run: bool,

    /// With --dry-run: also compile the moved workspace.
    ///
    /// Carries the move out on a throw-away copy of the workspace (without `target`, and inside
    /// REFAC_DRY_RUN_COPY_MAX_MB, default 500) and runs `cargo check --workspace --all-targets`
    /// there, as the real move does. The copy builds into a folder of its own, so this takes as
    /// long as a first build. The project is not touched.
    #[arg(long, requires = "dry_run")]
    pub(super) check: bool,

    /// Print one JSON document instead of text (an error is JSON on stderr).
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct RenameArgs {
    /// The root the language server loads (default: the current directory).
    ///
    /// TypeScript/JavaScript: the folder with the authoritative tsconfig.json. Kotlin: the Gradle
    /// root. Go: the folder with go.mod. Rust: the Cargo package or workspace root. Python: the
    /// pyright root. Dart: the package folder with pubspec.yaml.
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub(super) project_path: Option<std::path::PathBuf>,

    /// File that contains the symbol (absolute, or relative to --project-path).
    ///
    /// Its extension picks the language: ts tsx js jsx mts cts mjs cjs, kt, go, rs, py, dart.
    #[arg(long, value_name = "FILE", value_hint = ValueHint::FilePath, required_unless_present = "batch", conflicts_with = "batch")]
    pub(super) file: Option<std::path::PathBuf>,

    /// The symbol's current name, written exactly as in that file.
    #[arg(
        long,
        value_name = "NAME",
        required_unless_present = "batch",
        conflicts_with = "batch"
    )]
    pub(super) symbol: Option<String>,

    /// The new name. It must be a plain identifier and not a keyword of the language.
    #[arg(
        long,
        value_name = "NAME",
        required_unless_present = "batch",
        conflicts_with = "batch"
    )]
    pub(super) new_name: Option<String>,

    /// 1-based line that picks the symbol when the name means several in the file.
    ///
    /// Needed only when the command says "The name refers to several different symbols".
    #[arg(long, value_name = "N", conflicts_with = "batch")]
    pub(super) line: Option<u32>,

    /// 1-based byte column on --line (like `rg --column`), for a line that holds the name twice.
    #[arg(long, value_name = "N", requires = "line", conflicts_with = "batch")]
    pub(super) column: Option<u32>,

    /// Run several renames in one language-server session, all or nothing (JSON file, or `-` for stdin).
    ///
    /// The file holds a list: [{"file": "src/a.rs", "symbol": "old_a", "new_name": "new_a"}, ...];
    /// an entry may add "line" and "column". The server starts once, each rename is proven and
    /// written against the files the one before left, and one failure undoes the earlier ones.
    /// One language per batch (a mixed batch is refused, naming the entry). With --dry-run each entry
    /// is planned on the files as the ones before it would leave them. Replaces --file, --symbol
    /// and --new-name.
    #[arg(long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) batch: Option<std::path::PathBuf>,

    /// Plan and prove the rename, print it, and change no file.
    #[arg(long)]
    pub(super) dry_run: bool,

    /// Kotlin only: let a single Kotlin change through when REFAC_KOTLIN_BATCH_ONLY=1 refuses it.
    ///
    /// Every Kotlin command starts the language server and imports the Gradle build (30 to 40 s,
    /// 1.3 to 1.8 GB), however few files it changes, so a series of single changes is slow. With
    /// REFAC_KOTLIN_BATCH_ONLY=1 a single Kotlin rename is refused with the command that does several
    /// in one call; this argument says that the one change is all there is. It has no effect on
    /// other languages, on a batch, or when the variable is not set.
    #[arg(long)]
    pub(super) allow_single: bool,

    /// Print one JSON document instead of text (an error is JSON on stderr).
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Debug, Args)]
pub(super) struct CompletionsArgs {
    /// The shell to print completions for.
    #[arg(value_enum)]
    pub(super) shell: Shell,
}
