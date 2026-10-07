use super::edits::{FileEdits, LineIndex, RawEdit, RawFileEdits, collect_raw, ensure_editable};
use super::locate::Occurrence;
use super::session::{RpcError, Session};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::Path;
use url::Url;

/// Every text change needed to rename one symbol, sorted by file path.
pub struct Plan {
    pub files: Vec<FileEdits>,
}

impl Plan {
    pub fn edit_count(&self) -> usize {
        self.files.iter().map(|file| file.edits.len()).sum()
    }
}

/// One distinct symbol reachable from the textual matches.
struct Group {
    anchor: Occurrence,
    raw: RawFileEdits,
    /// Byte ranges this symbol covers in the target file.
    covered: Vec<(usize, usize)>,
}

/// Ask the engine which symbol each textual match names. Matches the engine
/// refuses are skipped; matches inside an earlier symbol's edit ranges are the
/// same symbol. More than one distinct symbol is an explicit ambiguity.
pub async fn plan_rename(
    session: &mut Session,
    file: &Path,
    text: &str,
    matches: &[Occurrence],
    new_name: &str,
    project_root: &Path,
) -> Result<Plan> {
    let uri = Url::from_file_path(file)
        .map_err(|_| anyhow::anyhow!("Invalid file path {}", file.display()))?
        .to_string();
    let index = LineIndex::new(text);
    let mut groups: Vec<Group> = Vec::new();
    let mut refusal: Option<String> = None;
    for occurrence in matches {
        if groups.iter().any(|group| {
            group
                .covered
                .iter()
                .any(|(s, e)| *s <= occurrence.offset && occurrence.offset < *e)
        }) {
            continue;
        }
        let position = json!({ "line": occurrence.line, "character": occurrence.column });
        let prepared = session
            .request(
                "textDocument/prepareRename",
                json!({ "textDocument": { "uri": uri }, "position": position }),
            )
            .await;
        match prepared {
            Ok(Value::Null) => {
                refusal.get_or_insert_with(|| {
                    "this position is not a renameable identifier".to_string()
                });
                continue;
            }
            Ok(_) => {}
            Err(error) => match error.downcast_ref::<RpcError>() {
                Some(rpc) => {
                    refusal.get_or_insert_with(|| rpc.message.clone());
                    continue;
                }
                None => return Err(error),
            },
        }
        let edit = session
            .request("textDocument/rename", json!({ "textDocument": { "uri": uri }, "position": position, "newName": new_name }))
            .await?;
        let raw = collect_raw(&edit)?;
        let covered = raw
            .iter()
            .filter(|(path, _)| path == file)
            .flat_map(|(_, edits)| edits.iter())
            .map(|edit| range_in(&index, text, edit))
            .collect::<Result<Vec<_>>>()?;
        groups.push(Group {
            anchor: *occurrence,
            raw,
            covered,
        });
    }
    match groups.len() {
        0 => bail!(
            "Cannot rename: {}",
            refusal.unwrap_or_else(|| "the engine found no renameable symbol".into())
        ),
        1 => build_plan(groups.remove(0).raw, project_root),
        _ => bail!("{}", ambiguity(text, &groups)),
    }
}

fn range_in(index: &LineIndex, text: &str, edit: &RawEdit) -> Result<(usize, usize)> {
    Ok((
        index.offset(text, edit.range[0], edit.range[1])?,
        index.offset(text, edit.range[2], edit.range[3])?,
    ))
}

fn build_plan(raw: RawFileEdits, project_root: &Path) -> Result<Plan> {
    let mut files = Vec::new();
    for (path, edits) in raw {
        ensure_editable(&path, project_root)?;
        files.push(FileEdits::build(path, &edits)?);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Plan { files })
}

fn ambiguity(text: &str, groups: &[Group]) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut message = String::from(
        "The name refers to several different symbols in this file. Pass --line (and --column) to choose one:",
    );
    for group in groups {
        let count: usize = group.raw.iter().map(|(_, edits)| edits.len()).sum();
        let source = lines
            .get(group.anchor.line as usize)
            .map_or("", |line| line.trim());
        message.push_str(&format!(
            "\n  {}:{}  {}  ({} in {})",
            group.anchor.line + 1,
            group.anchor.column + 1,
            source.chars().take(100).collect::<String>(),
            plural(count, "edit"),
            plural(group.raw.len(), "file")
        ));
    }
    message
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}
