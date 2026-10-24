//! What every language backend of `refac rename` takes and returns.

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct RenameRequest {
    pub project_path: PathBuf,
    pub file: PathBuf,
    pub symbol: String,
    pub new_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub dry_run: bool,
    /// TypeScript only: also rename inside `.astro` files, by letting the
    /// engine run the project's Astro content mapper. That executes code from
    /// the project's `node_modules`, so it is never the default.
    pub astro: bool,
}

#[derive(Debug)]
pub struct RenameReport {
    /// Edited files relative to the project, with their edit counts.
    pub files: Vec<(PathBuf, usize)>,
    pub edits: usize,
    pub dry_run: bool,
    /// Things the user should know that are not edits: a file renamed along
    /// with its class, old names left in files refac does not edit.
    pub notes: Vec<String>,
}
