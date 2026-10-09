use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::{Path, PathBuf};
use url::Url;

/// Maps LSP line/character positions to byte offsets. The session negotiates
/// UTF-8 positions, so `character` counts bytes within its line.
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut starts = vec![0];
        for (index, byte) in bytes.iter().enumerate() {
            // A lone carriage return also ends a line; CRLF is one break.
            let ends_line =
                *byte == b'\n' || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n'));
            if ends_line {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    pub fn offset(&self, text: &str, line: u64, character: u64) -> Result<usize> {
        let start = *self
            .starts
            .get(line as usize)
            .with_context(|| format!("Position line {line} is past the end of the file"))?;
        let end = self
            .starts
            .get(line as usize + 1)
            .copied()
            .unwrap_or(text.len());
        let offset = start + character as usize;
        if offset > end || !text.is_char_boundary(offset) {
            bail!("Position {line}:{character} is not a valid UTF-8 boundary in its line");
        }
        Ok(offset)
    }

    /// Zero-based line and byte column of an offset.
    pub fn position(&self, offset: usize) -> (u32, u32) {
        let line = self.starts.partition_point(|start| *start <= offset) - 1;
        (line as u32, (offset - self.starts[line]) as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub new_text: String,
}

/// One edit as the server sent it: zero-based line/character range plus text.
pub struct RawEdit {
    pub range: [u64; 4],
    pub new_text: String,
}

pub type RawFileEdits = Vec<(PathBuf, Vec<RawEdit>)>;

/// All edits for one file with its text before and after, both BOM-free.
pub struct FileEdits {
    pub path: PathBuf,
    pub bom: bool,
    pub old: String,
    pub new: String,
    pub edits: Vec<TextEdit>,
}

/// Read the per-file edits of an LSP `WorkspaceEdit`. File create, rename and
/// delete operations are rejected: a symbol rename only changes text.
pub fn collect_raw(edit: &Value) -> Result<RawFileEdits> {
    let mut files: RawFileEdits = Vec::new();
    let mut add = |uri: &str, edits: &Value| -> Result<()> {
        let path = Url::parse(uri)
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .with_context(|| format!("Rename edit targets a non-file URI: {uri}"))?;
        let list = edits.as_array().context("Rename edits are not a list")?;
        let parsed = list
            .iter()
            .map(parse_raw_edit)
            .collect::<Result<Vec<_>>>()?;
        files.push((path, parsed));
        Ok(())
    };
    if let Some(changes) = edit.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            add(uri, edits)?;
        }
    }
    if let Some(changes) = edit.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            if let Some(kind) = change.get("kind").and_then(Value::as_str) {
                bail!("Rename returned a file {kind} operation, which is unsupported");
            }
            let uri = change["textDocument"]["uri"]
                .as_str()
                .context("documentChanges entry has no URI")?;
            add(uri, &change["edits"])?;
        }
    }
    files.retain(|(_, edits)| !edits.is_empty());
    Ok(files)
}

fn parse_raw_edit(edit: &Value) -> Result<RawEdit> {
    let number = |value: &Value| value.as_u64().context("Rename edit range is not numeric");
    let range = &edit["range"];
    Ok(RawEdit {
        range: [
            number(&range["start"]["line"])?,
            number(&range["start"]["character"])?,
            number(&range["end"]["line"])?,
            number(&range["end"]["character"])?,
        ],
        new_text: edit["newText"]
            .as_str()
            .context("Rename edit has no newText")?
            .to_string(),
    })
}

/// A rename may only change project source: never `node_modules`, never a path
/// outside the project (a referencing project, for example).
pub fn ensure_editable(path: &Path, project_root: &Path) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Rename edit targets a missing file: {}", path.display()))?;
    if !canonical.starts_with(project_root) {
        bail!(
            "Rename would edit {} outside the project {}. Nothing was changed. Run the rename from a project path whose tsconfig owns every usage.",
            canonical.display(),
            project_root.display()
        );
    }
    if canonical
        .components()
        .any(|part| part.as_os_str() == "node_modules")
    {
        bail!(
            "Rename would edit a file inside node_modules: {}",
            canonical.display()
        );
    }
    Ok(())
}

impl FileEdits {
    pub fn build(path: PathBuf, raw: &[RawEdit]) -> Result<Self> {
        let text = crate::drivers::symbol::view::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        let (bom, old) = match text.strip_prefix('\u{FEFF}') {
            Some(rest) => (true, rest.to_string()),
            None => (false, text),
        };
        let index = LineIndex::new(&old);
        let mut edits = Vec::new();
        for edit in raw {
            edits.push(TextEdit {
                start: index.offset(&old, edit.range[0], edit.range[1])?,
                end: index.offset(&old, edit.range[2], edit.range[3])?,
                new_text: edit.new_text.clone(),
            });
        }
        edits.sort_by_key(|edit| edit.start);
        if edits.windows(2).any(|pair| pair[0].end > pair[1].start) {
            bail!("Rename returned overlapping edits in {}", path.display());
        }
        let new = apply(&old, &edits);
        Ok(Self {
            path,
            bom,
            old,
            new,
            edits,
        })
    }

    /// Offset in the renamed text where edit `index` begins.
    pub fn new_start(&self, index: usize) -> usize {
        let shift: isize = self.edits[..index]
            .iter()
            .map(|edit| edit.new_text.len() as isize - (edit.end - edit.start) as isize)
            .sum();
        (self.edits[index].start as isize + shift) as usize
    }
}

pub fn apply(text: &str, edits: &[TextEdit]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for edit in edits {
        out.push_str(&text[cursor..edit.start]);
        out.push_str(&edit.new_text);
        cursor = edit.end;
    }
    out.push_str(&text[cursor..]);
    out
}

/// Where the new identifier sits inside one edit's replacement text, for the
/// replacement shapes TypeScript produces. `None` means an unknown shape.
/// Shorthand properties and export specifiers keep the old public name, so the
/// new name is not always at offset 0.
pub fn identifier_offset(old: &str, new_text: &str, new_name: &str) -> Option<usize> {
    if new_text == new_name {
        return Some(0);
    }
    if new_text == format!("{old}: {new_name}") {
        return Some(old.len() + 2); // `{ total }` becomes `{ total: renamed }`
    }
    if new_text == format!("{new_name}: {old}") || new_text == format!("{new_name} as {old}") {
        return Some(0); // `{ renamed: total }`, `export { renamed as total }`
    }
    let quote = old
        .chars()
        .next()
        .filter(|quote| matches!(quote, '"' | '\''))?;
    (new_text == format!("{quote}{new_name}{quote}")).then_some(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_columns_count_bytes_and_crlf_is_one_break() {
        let text = "é😀 = 1;\r\nlet total = 2;\n";
        let index = LineIndex::new(text);
        assert_eq!(index.offset(text, 1, 4).unwrap(), "é😀 = 1;\r\nlet ".len());
        assert_eq!(index.position("é😀 = 1;\r\nlet ".len()), (1, 4));
        assert!(
            index.offset(text, 0, 1).is_err(),
            "column inside a multibyte char"
        );
        assert!(index.offset(text, 9, 0).is_err(), "line past the end");
    }

    #[test]
    fn applies_sorted_edits_and_tracks_new_starts() {
        let edits = vec![
            TextEdit {
                start: 6,
                end: 11,
                new_text: "grandTotal".into(),
            },
            TextEdit {
                start: 19,
                end: 24,
                new_text: "total: grandTotal".into(),
            },
        ];
        let old = "const total = 1; { total }";
        assert_eq!(
            apply(old, &edits),
            "const grandTotal = 1; { total: grandTotal }"
        );
        let file = FileEdits {
            path: PathBuf::new(),
            bom: false,
            old: old.into(),
            new: apply(old, &edits),
            edits,
        };
        assert_eq!(file.new_start(1), 24);
    }

    #[test]
    fn recognises_typescript_replacement_shapes() {
        assert_eq!(identifier_offset("total", "grand", "grand"), Some(0));
        assert_eq!(identifier_offset("total", "total: tot", "tot"), Some(7));
        assert_eq!(identifier_offset("total", "tot: total", "tot"), Some(0));
        assert_eq!(
            identifier_offset("total", "total2 as total", "total2"),
            Some(0)
        );
        assert_eq!(
            identifier_offset("\"total\"", "\"grand\"", "grand"),
            Some(1)
        );
        assert_eq!(identifier_offset("total", "something else", "grand"), None);
    }

    #[test]
    fn rejects_resource_operations_and_non_file_uris() {
        let create =
            serde_json::json!({"documentChanges": [{"kind": "create", "uri": "file:///x"}]});
        assert!(collect_raw(&create).is_err());
        let web = serde_json::json!({"changes": {"https://example.com/a.ts": []}});
        assert!(collect_raw(&web).is_err());
    }
}
