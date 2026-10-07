//! The JSON documents printed with `--json`, one shape per outcome.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct MoveSuccessOutput<'a> {
    pub(super) status: &'static str,
    pub(super) operation: &'static str,
    pub(super) project_path: Option<&'a str>,
    pub(super) source_path: &'a [String],
    pub(super) target_path: &'a [String],
    pub(super) result: &'a str,
}

#[derive(Debug, Serialize)]
pub(super) struct MoveModuleSuccessOutput<'a> {
    pub(super) status: &'static str,
    pub(super) operation: &'static str,
    pub(super) project_path: &'a str,
    pub(super) source_module: &'a str,
    pub(super) target_module: &'a str,
    pub(super) moved_paths: usize,
    pub(super) edited_files: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct RenameSuccessOutput<'a> {
    pub(super) status: &'static str,
    pub(super) operation: &'static str,
    pub(super) project_path: &'a str,
    pub(super) file: &'a str,
    pub(super) symbol: &'a str,
    pub(super) new_name: &'a str,
    pub(super) dry_run: bool,
    pub(super) edits: usize,
    pub(super) edited_files: usize,
    pub(super) files: Vec<RenamedFile>,
    pub(super) notes: &'a [String],
}

#[derive(Debug, Serialize)]
pub(super) struct RenamedFile {
    pub(super) path: String,
    pub(super) edits: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ErrorOutput<'a> {
    pub(super) status: &'static str,
    pub(super) error: &'a str,
}
