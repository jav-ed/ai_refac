use super::edits::{FileEdits, LineIndex, identifier_offset};
use super::plan::Plan;
use super::session::Session;
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

/// TypeScript's rename does not notice when the new name already exists in
/// scope: a clash compiles into merged declarations, and a shadowing name
/// silently rebinds other usages. Prove the rename is faithful before any file
/// changes: show the engine the renamed text in memory, ask for the references
/// of the renamed declaration, and require exactly one new-name reference per
/// edit. References spelled differently, such as `renamed as total` aliases
/// that keep a public export name, are not counted.
pub async fn verify(session: &mut Session, plan: &Plan, new_name: &str) -> Result<()> {
    for file in &plan.files {
        session.sync_document(&file.path, &file.new).await?;
    }
    let (anchor_file, anchor_offset) = pick_anchor(plan, new_name)?;
    let (line, character) = LineIndex::new(&anchor_file.new).position(anchor_offset);
    let uri = Url::from_file_path(&anchor_file.path)
        .map_err(|_| anyhow::anyhow!("Invalid file path {}", anchor_file.path.display()))?
        .to_string();
    let locations = session
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
                "context": { "includeDeclaration": true },
            }),
        )
        .await?;

    let mut found: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for location in locations
        .as_array()
        .context("References answer is not a list")?
    {
        let path = Url::parse(location["uri"].as_str().context("Reference without URI")?)
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .context("Reference with a non-file URI")?;
        let text = text_after_rename(plan, &path)?;
        let index = LineIndex::new(&text);
        let range = &location["range"];
        let number =
            |value: &serde_json::Value| value.as_u64().context("Reference range is not numeric");
        let start = index.offset(
            &text,
            number(&range["start"]["line"])?,
            number(&range["start"]["character"])?,
        )?;
        let end = index.offset(
            &text,
            number(&range["end"]["line"])?,
            number(&range["end"]["character"])?,
        )?;
        if text[start..end] == *new_name {
            *found.entry(path).or_default() += 1;
        }
    }
    let expected: BTreeMap<PathBuf, usize> = plan
        .files
        .iter()
        .map(|file| (file.path.clone(), file.edits.len()))
        .collect();
    if found != expected {
        bail!(
            "Rename to `{new_name}` is not faithful: the new name clashes with or shadows another symbol in scope, so references would change meaning. Nothing was changed.\n{}",
            describe(&expected, &found)
        );
    }
    Ok(())
}

/// The first edit whose replacement shape is known. Plain edits, where the
/// replacement is exactly the new name, come first because they are
/// unambiguous.
fn pick_anchor<'a>(plan: &'a Plan, new_name: &str) -> Result<(&'a FileEdits, usize)> {
    let shapes = plan.files.iter().flat_map(|file| {
        file.edits
            .iter()
            .enumerate()
            .filter_map(move |(index, edit)| {
                let old = &file.old[edit.start..edit.end];
                identifier_offset(old, &edit.new_text, new_name).map(|inside| {
                    (
                        file,
                        file.new_start(index) + inside,
                        edit.new_text == new_name,
                    )
                })
            })
    });
    let all: Vec<_> = shapes.collect();
    let chosen = all.iter().find(|(_, _, plain)| *plain).or(all.first());
    match chosen {
        Some((file, offset, _)) => Ok((file, *offset)),
        None => bail!(
            "The engine returned replacement text this tool cannot verify; refusing to rename without verification. Nothing was changed."
        ),
    }
}

fn text_after_rename(plan: &Plan, path: &Path) -> Result<String> {
    if let Some(file) = plan.files.iter().find(|file| file.path == path) {
        return Ok(file.new.clone());
    }
    let text = crate::drivers::symbol::view::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    Ok(text
        .strip_prefix('\u{FEFF}')
        .map_or(text.clone(), str::to_string))
}

fn describe(expected: &BTreeMap<PathBuf, usize>, found: &BTreeMap<PathBuf, usize>) -> String {
    let mut lines = Vec::new();
    for path in expected
        .keys()
        .chain(found.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let want = expected.get(path).copied().unwrap_or(0);
        let got = found.get(path).copied().unwrap_or(0);
        if want != got {
            lines.push(format!(
                "  {}: {want} renamed usages expected, {got} resolve to the new name",
                path.display()
            ));
        }
    }
    lines.join("\n")
}
