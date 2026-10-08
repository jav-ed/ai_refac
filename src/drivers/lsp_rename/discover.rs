//! Planning a symbol rename: which symbol the name stands for, and every text
//! change and file rename that renaming it takes. Nothing is written here.

use super::edits::{Change, FileEdits, PlannedFile, parse_changes, plan_files};
use super::family;
use super::language::Language;
use super::server::RenameServer;
use crate::drivers::lsp_session::RpcError;
use crate::drivers::lsp::text::TextIndex;
use crate::drivers::symbol_scan::Occurrence;
use anyhow::{Context, Result, bail};
use lsp_types::{Range, TextEdit};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use url::Url;

/// A file with the text edits that rename the symbol in it.
pub struct EditedFile {
    pub file: PlannedFile,
    pub edits: Vec<TextEdit>,
}

pub struct RenamePlan {
    pub files: Vec<EditedFile>,
    /// Files the server wants moved with the symbol: (current path, new path).
    pub moves: Vec<(PathBuf, PathBuf)>,
}

impl RenamePlan {
    pub fn edit_count(&self) -> usize {
        self.files.iter().map(|file| file.edits.len()).sum()
    }
}

/// A place the symbol is declared or used.
#[derive(Debug, Clone)]
pub struct Reference {
    pub path: PathBuf,
    pub range: Range,
}

/// One distinct symbol behind the name, found from one occurrence.
pub struct Candidate {
    pub anchor: Occurrence,
    /// Start of the reference that contains the anchor, in the target file.
    pub anchor_start: usize,
    pub references: Vec<Reference>,
    pub plan: RenamePlan,
}

/// Ask the server which symbol each textual match names. Matches it refuses
/// are skipped; matches inside an earlier symbol's references are the same
/// symbol. More than one distinct symbol is an explicit ambiguity.
pub async fn discover(
    server: &mut dyn RenameServer,
    language: &dyn Language,
    file: &Path,
    text: &str,
    occurrences: &[Occurrence],
    symbol: &str,
    new_name: &str,
) -> Result<Candidate> {
    let uri = file_uri(file)?;
    let index = TextIndex::new(text);
    let mut candidates: Vec<(Candidate, Vec<(usize, usize)>)> = Vec::new();
    let mut refusal: Option<String> = None;
    let mut symbol_refusal: Option<String> = None;
    for occurrence in occurrences {
        if candidates.iter().any(|(_, covered)| {
            covered
                .iter()
                .any(|(start, end)| *start <= occurrence.offset && occurrence.offset < *end)
        }) {
            continue;
        }
        let position = index.position(occurrence.offset);
        let at = json!({ "textDocument": { "uri": uri }, "position": position });
        match refuse_or(
            server
                .request("textDocument/prepareRename", at.clone())
                .await,
        )? {
            Ok(Value::Null) => {
                refusal.get_or_insert_with(|| match language.not_a_symbol_hint() {
                    Some(hint) => format!("this position is not a renameable symbol. {hint}"),
                    None => "this position is not a renameable symbol".to_string(),
                });
                continue;
            }
            Ok(_) => {}
            Err(message) => {
                refusal.get_or_insert(message);
                continue;
            }
        }
        let mut references_request = at.clone();
        references_request["context"] = json!({ "includeDeclaration": true });
        let references = match refuse_or(
            server
                .request("textDocument/references", references_request)
                .await,
        )? {
            Ok(references) => parse_references(&references)?,
            Err(message) => {
                refusal.get_or_insert(message);
                continue;
            }
        };
        let mut rename_request = at.clone();
        rename_request["newName"] = json!(new_name);
        let answer = match refuse_or(server.request("textDocument/rename", rename_request).await)? {
            Ok(answer) => answer,
            Err(message) => {
                // The server knew the symbol and still refused: that reason
                // (a name clash, a broken interface) beats "not a symbol".
                symbol_refusal.get_or_insert(message);
                continue;
            }
        };
        let covered = covered_in(&references, file, &index)?;
        let anchor_start = covered
            .iter()
            .find(|(start, end)| *start <= occurrence.offset && occurrence.offset < *end)
            .map(|(start, _)| *start)
            .context("The server's references do not include the position it was asked about")?;
        let mut changes = parse_changes(&answer)?;
        if language.renames_overrides() {
            changes =
                family::with_overrides(server, &at, &references, symbol, new_name, changes).await?;
        }
        let plan = build_plan(changes, language)?;
        candidates.push((
            Candidate {
                anchor: *occurrence,
                anchor_start,
                references,
                plan,
            },
            covered,
        ));
    }
    match candidates.len() {
        0 => bail!(
            "Cannot rename: {}",
            symbol_refusal
                .or(refusal)
                .unwrap_or_else(|| "the server found no renameable symbol".into())
        ),
        1 => Ok(candidates.remove(0).0),
        _ => bail!(
            "{}",
            ambiguity(
                text,
                &candidates
                    .iter()
                    .map(|(candidate, _)| candidate)
                    .collect::<Vec<_>>()
            )
        ),
    }
}

/// An error answer from the server is a refusal to report; a broken
/// connection is not.
pub(super) fn refuse_or(answer: Result<Value>) -> Result<std::result::Result<Value, String>> {
    match answer {
        Ok(value) => Ok(Ok(value)),
        Err(error) => match error.downcast_ref::<RpcError>() {
            Some(rpc) => Ok(Err(rpc.message.clone())),
            None => Err(error),
        },
    }
}

pub(super) fn parse_references(answer: &Value) -> Result<Vec<Reference>> {
    let Some(locations) = answer.as_array() else {
        bail!("The server answered the references request with {answer}");
    };
    locations
        .iter()
        .map(|location| {
            let uri = location["uri"]
                .as_str()
                .context("A reference without a uri")?;
            let path = Url::parse(uri)
                .ok()
                .and_then(|url| url.to_file_path().ok())
                .with_context(|| format!("A reference outside the local files: {uri}"))?;
            let range = serde_json::from_value(location["range"].clone())
                .context("A reference without a range")?;
            Ok(Reference { path, range })
        })
        .collect()
}

/// Byte ranges of the references that lie in `file`.
fn covered_in(
    references: &[Reference],
    file: &Path,
    index: &TextIndex,
) -> Result<Vec<(usize, usize)>> {
    references
        .iter()
        .filter(|reference| reference.path == file)
        .map(|reference| {
            Ok((
                index.offset(reference.range.start)?,
                index.offset(reference.range.end)?,
            ))
        })
        .collect()
}

fn build_plan(changes: Vec<Change>, language: &dyn Language) -> Result<RenamePlan> {
    let mut moves: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut edits: Vec<FileEdits> = Vec::new();
    for change in changes {
        match change {
            Change::Edit(file) => {
                // Edits sent after a file rename address the new path.
                let path = moves
                    .iter()
                    .find(|(_, to)| *to == file.path)
                    .map_or(file.path, |(from, _)| from.clone());
                if edits.iter().any(|known| known.path == path) {
                    bail!(
                        "The server edits {} in two separate steps, which refac does not support",
                        path.display()
                    );
                }
                edits.push(FileEdits {
                    path,
                    edits: file.edits,
                });
            }
            Change::Other { kind, path } => {
                let advice = language
                    .refuse_file_operations()
                    .unwrap_or("refac does not apply create or delete file operations");
                bail!(
                    "Renaming this symbol would also {kind} {}. {advice}",
                    path.display()
                );
            }
            Change::Rename { from, to } => {
                if let Some(advice) = language.refuse_file_operations() {
                    bail!(
                        "Renaming this symbol would also rename {} to {}. {advice}",
                        from.display(),
                        to.display()
                    );
                }
                if to.exists() {
                    bail!(
                        "The rename would move {} onto {}, which already exists",
                        from.display(),
                        to.display()
                    );
                }
                moves.push((from, to));
            }
        }
    }
    let originals: Vec<Vec<TextEdit>> = edits.iter().map(|file| file.edits.clone()).collect();
    let planned = plan_files(edits, Path::to_path_buf)?;
    let files = planned
        .into_iter()
        .zip(originals)
        .map(|(file, edits)| EditedFile { file, edits })
        .collect();
    Ok(RenamePlan { files, moves })
}

/// Every place the symbol at `position` is declared or used.
pub(super) async fn references_at(
    server: &mut dyn RenameServer,
    path: &Path,
    position: lsp_types::Position,
) -> Result<Vec<Reference>> {
    let answer = server
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": file_uri(path)? },
                "position": position,
                "context": { "includeDeclaration": true },
            }),
        )
        .await?;
    parse_references(&answer)
}

pub fn file_uri(path: &Path) -> Result<String> {
    Ok(Url::from_file_path(path)
        .map_err(|_| anyhow::anyhow!("Invalid file path {}", path.display()))?
        .to_string())
}

fn ambiguity(text: &str, candidates: &[&Candidate]) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut message = String::from(
        "The name refers to several different symbols in this file. Pass --line (and --column) to choose one:",
    );
    for candidate in candidates {
        let source = lines
            .get(candidate.anchor.line as usize)
            .map_or("", |line| line.trim());
        message.push_str(&format!(
            "\n  {}:{}  {}  ({} in {})",
            candidate.anchor.line + 1,
            candidate.anchor.column + 1,
            source.chars().take(100).collect::<String>(),
            plural(candidate.plan.edit_count(), "edit"),
            plural(candidate.plan.files.len(), "file")
        ));
    }
    message
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

#[cfg(test)]
mod tests;
