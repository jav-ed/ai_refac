//! Helpers for the rename tests that run a real language server. Those tests
//! are `#[ignore]`d; running them without the server is a setup mistake, so
//! they fail loudly with what refac itself says about the missing server.

use refac::drivers::symbol_rename::{RenameReport, RenameRequest};
use refac::logic::rename::{handle_rename, handle_rename_batch};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Stop with refac's own explanation when the server of `language` is not
/// installed.
pub fn require_server(language: &str) {
    let server = refac::servers::for_language(language).expect("a known language");
    let here = std::env::current_dir().expect("a working directory");
    if let Err(not_found) = refac::servers::locate(server, &here) {
        panic!("{not_found}");
    }
}

pub fn request(project: &Path, file: &str, symbol: &str, new_name: &str) -> RenameRequest {
    RenameRequest {
        project_path: project.to_path_buf(),
        file: file.into(),
        symbol: symbol.to_string(),
        new_name: new_name.to_string(),
        line: None,
        column: None,
        dry_run: false,
    }
}

pub fn at_line(mut request: RenameRequest, line: u32) -> RenameRequest {
    request.line = Some(line);
    request
}

pub async fn rename(request: RenameRequest) -> RenameReport {
    handle_rename(request)
        .await
        .unwrap_or_else(|error| panic!("the rename failed: {error:#}"))
}

/// Several renames in one server session; each must succeed.
pub async fn batch(requests: Vec<RenameRequest>) -> Vec<RenameReport> {
    handle_rename_batch(requests)
        .await
        .unwrap_or_else(|error| panic!("the batch failed: {error:#}"))
}

/// The error of a batch that must be refused.
pub async fn refused_batch(requests: Vec<RenameRequest>) -> String {
    match handle_rename_batch(requests).await {
        Ok(reports) => panic!(
            "the batch should be refused, but changed {:?}",
            reports
                .iter()
                .map(|report| &report.files)
                .collect::<Vec<_>>()
        ),
        Err(error) => format!("{error:#}"),
    }
}

/// The error of a rename that must be refused.
pub async fn refused(request: RenameRequest) -> String {
    match handle_rename(request).await {
        Ok(report) => panic!(
            "the rename should be refused, but changed {:?}",
            report.files
        ),
        Err(error) => format!("{error:#}"),
    }
}

/// Every file below `root` with its bytes, to prove that a refused rename left
/// the project as it was. Build output and tool state are not part of it.
pub fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy();
            if path.is_dir() {
                let skipped = [
                    ".git",
                    "target",
                    "node_modules",
                    "__pycache__",
                    ".mypy_cache",
                    ".venv",
                    "build",
                ];
                if !skipped.contains(&name.as_ref()) {
                    walk(root, &path, out);
                }
            } else if name != "Cargo.lock" {
                // Cargo.lock is written by rust-analyzer's own `cargo metadata`.
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// The project is exactly as `before` recorded it. Names the files that differ
/// instead of dumping their bytes.
pub fn assert_unchanged(root: &Path, before: &BTreeMap<String, Vec<u8>>) {
    let after = snapshot(root);
    let mut differing: Vec<&String> = before
        .keys()
        .chain(after.keys())
        .filter(|name| before.get(*name) != after.get(*name))
        .collect();
    differing.sort();
    differing.dedup();
    assert!(
        differing.is_empty(),
        "a refused rename changed files: {differing:?}"
    );
}
