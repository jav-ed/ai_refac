//! What every language backend of `refac rename` takes and returns.

use std::path::PathBuf;

#[derive(Debug)]
pub struct RenameRequest {
    pub project_path: PathBuf,
    pub file: PathBuf,
    pub symbol: String,
    pub new_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub dry_run: bool,
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
