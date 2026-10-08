use super::*;
use crate::drivers::lsp_rename::discover::EditedFile;
use crate::drivers::lsp_rename::edits::PlannedFile;
use crate::drivers::lsp::text::apply_text_edits;
use async_trait::async_trait;
use lsp_types::{Position, Range, TextEdit};
use serde_json::Value;

const FILE: &str = "/tmp/refac-verify-tests/a.src";
const ROOT: &str = "/tmp/refac-verify-tests";

fn range(line: u32, from: u32, to: u32) -> Range {
    Range {
        start: Position::new(line, from),
        end: Position::new(line, to),
    }
}

fn edit(line: u32, from: u32, to: u32, text: &str) -> TextEdit {
    TextEdit {
        range: range(line, from, to),
        new_text: text.to_string(),
    }
}

fn reference(line: u32, from: u32, to: u32) -> Reference {
    Reference {
        path: PathBuf::from(FILE),
        range: range(line, from, to),
    }
}

fn edited(before: &str, edits: Vec<TextEdit>) -> EditedFile {
    let text = apply_text_edits(before, edits.clone()).unwrap();
    EditedFile {
        file: PlannedFile {
            path: PathBuf::from(FILE),
            bytes: text.clone().into_bytes(),
            before: before.to_string(),
            text,
        },
        edits,
    }
}

fn plan(before: &str, edits: Vec<TextEdit>) -> RenamePlan {
    RenamePlan {
        files: vec![edited(before, edits)],
        moves: Vec::new(),
    }
}

fn group(references: Vec<Reference>) -> Group {
    Group {
        path: PathBuf::from(FILE),
        start: 3,
        references,
    }
}

/// A server that answers every references request with one fixed list.
struct Scripted {
    answer: Value,
}

#[async_trait]
impl RenameServer for Scripted {
    async fn request(&mut self, _method: &str, _params: Value) -> Result<Value> {
        Ok(self.answer.clone())
    }

    async fn sync_document(&mut self, _path: &Path, _text: &str) -> Result<()> {
        Ok(())
    }

    async fn shutdown(self: Box<Self>) {}
}

fn locations(places: &[(u32, u32)]) -> Value {
    serde_json::json!(
        places
            .iter()
            .map(|(line, column)| serde_json::json!({
                "uri": file_uri(Path::new(FILE)).unwrap(),
                "range": {
                    "start": { "line": line, "character": column },
                    "end": { "line": line, "character": column + 1 },
                },
            }))
            .collect::<Vec<_>>()
    )
}

#[test]
fn an_edit_touches_a_reference_when_it_overlaps_or_inserts_inside_it() {
    // The reference spans 8..16.
    assert!(touches(8, 16, 8, 16), "the whole name");
    assert!(touches(8, 9, 8, 16), "a letter at the start (minimal diff)");
    assert!(touches(10, 15, 8, 16), "letters in the middle");
    assert!(touches(16, 16, 8, 16), "an insertion at the end");
    assert!(touches(4, 12, 8, 16), "an edit that starts before it");
}

#[test]
fn an_edit_next_to_a_reference_does_not_touch_it() {
    assert!(!touches(0, 8, 8, 16), "ends where the name starts");
    assert!(!touches(16, 20, 8, 16), "starts where the name ends");
    assert!(!touches(20, 20, 8, 16), "an insertion after it");
    assert!(!touches(2, 2, 8, 16), "an insertion before it");
}

#[test]
fn an_edit_before_a_reference_moves_it_and_one_at_its_start_does_not() {
    let before = "let a = 1; let area = 2;\n";
    let index = TextIndex::new(before);
    let start = 15; // `area`
    // "a" -> "alpha" before it: five letters instead of one.
    let moved = shifted(&index, &[edit(0, 4, 5, "alpha")], start).unwrap();
    assert_eq!(moved, start + 4);
    // The identifier's own edit starts at `start` and does not move it.
    let own = shifted(&index, &[edit(0, 15, 19, "surface")], start).unwrap();
    assert_eq!(own, start);
    // An insertion exactly at the start (a prefix) stays before the name.
    let insertion = shifted(&index, &[edit(0, 15, 15, "self.")], start).unwrap();
    assert_eq!(insertion, start);
    // An edit after the reference changes nothing about it.
    let after = shifted(&index, &[edit(0, 20, 21, "xx")], start).unwrap();
    assert_eq!(after, start);
}

#[test]
fn expected_sites_follow_the_edits_to_where_the_references_will_be() {
    let before = "area(); area();\n";
    let plan = plan(
        before,
        vec![edit(0, 0, 4, "surface"), edit(0, 8, 12, "surface")],
    );
    let sites = expected_sites(&[reference(0, 0, 4), reference(0, 8, 12)], &plan).unwrap();
    // "surface(); surface();": the second one moved by three letters.
    let expected: BTreeSet<Site> = [(PathBuf::from(FILE), 0, 0), (PathBuf::from(FILE), 0, 11)]
        .into_iter()
        .collect();
    assert_eq!(sites, expected);
}

#[test]
fn a_reference_that_the_server_did_not_edit_is_an_unfaithful_answer() {
    let before = "area(); area();\n";
    // Only the first call is edited; the second is listed but forgotten.
    let plan = plan(before, vec![edit(0, 0, 4, "surface")]);
    let group = group(vec![reference(0, 0, 4), reference(0, 8, 12)]);
    let error =
        check_every_reference_is_edited(&plan, &group, "area", Path::new(ROOT)).unwrap_err();
    assert!(is_unfaithful(&error), "{error:#}");
    let text = error.to_string();
    assert!(text.contains("leaves 1 of the 2 places"), "{text}");
    assert!(text.contains("a.src:1"), "{text}");
}

#[test]
fn minimal_diff_edits_inside_the_name_count_as_edited() {
    let before = "decorate();\n";
    // "decorate" -> "embellish" as three small edits, like the Kotlin server.
    let plan = plan(
        before,
        vec![
            edit(0, 0, 1, "em"),
            edit(0, 2, 7, "bellis"),
            edit(0, 8, 8, "h"),
        ],
    );
    let group = group(vec![reference(0, 0, 8)]);
    assert!(check_every_reference_is_edited(&plan, &group, "decorate", Path::new(ROOT)).is_ok());
}

#[test]
fn a_reference_that_spells_something_else_needs_no_edit() {
    // `Self` stands for the type and keeps its spelling when the type is renamed.
    let before = "impl Shape { fn new() -> Self { Self } }\n";
    let plan = plan(before, vec![edit(0, 5, 10, "Figure")]);
    let group = group(vec![reference(0, 5, 10), reference(0, 29, 33)]);
    assert!(check_every_reference_is_edited(&plan, &group, "Shape", Path::new(ROOT)).is_ok());
}

/// A dependency in the package cache that uses the symbol: no server edits
/// it, so the rename can never be complete, and asking again cannot help.
#[test]
fn a_usage_in_a_file_outside_the_project_is_named_and_is_not_retried() {
    let project = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let outside = elsewhere.path().join("dependency.src");
    std::fs::write(&outside, "area();\n").unwrap();
    let before = "area();\n";
    let plan = plan(before, vec![edit(0, 0, 4, "surface")]);
    let group = group(vec![
        reference(0, 0, 4),
        Reference {
            path: outside.clone(),
            range: range(0, 0, 4),
        },
    ]);
    let error = check_every_reference_is_edited(&plan, &group, "area", project.path()).unwrap_err();
    assert!(!is_unfaithful(&error), "{error:#}");
    let text = error.to_string();
    assert!(
        text.contains("used in 1 places in files outside the project folder"),
        "{text}"
    );
    assert!(text.contains("dependency.src:1"), "{text}");
    assert!(text.contains("Nothing was changed"), "{text}");
}

#[tokio::test]
async fn the_usages_after_the_rename_must_be_the_old_ones_carried_through_the_edits() {
    let before = "area(); area();\n";
    let plan = plan(
        before,
        vec![edit(0, 0, 4, "surface"), edit(0, 8, 12, "surface")],
    );
    let group = group(vec![reference(0, 0, 4), reference(0, 8, 12)]);

    let mut faithful = Scripted {
        answer: locations(&[(0, 0), (0, 11)]),
    };
    check_group(&mut faithful, &plan, &group).await.unwrap();
}

#[tokio::test]
async fn a_usage_that_stopped_referring_to_the_symbol_is_named() {
    let before = "area(); area();\n";
    let plan = plan(
        before,
        vec![edit(0, 0, 4, "surface"), edit(0, 8, 12, "surface")],
    );
    let group = group(vec![reference(0, 0, 4), reference(0, 8, 12)]);

    // After the rename the server finds only the first usage: the second now
    // reaches another declaration.
    let mut lost = Scripted {
        answer: locations(&[(0, 0)]),
    };
    let error = check_group(&mut lost, &plan, &group).await.unwrap_err();
    assert!(is_unfaithful(&error));
    let text = error.to_string();
    assert!(text.contains("(2 before, 1 after)"), "{text}");
    assert!(
        text.contains("Usages that no longer refer to the symbol:\n  a.src:1:12"),
        "{text}"
    );
}

#[tokio::test]
async fn a_usage_that_started_referring_to_the_symbol_is_named() {
    let before = "area();\n";
    let plan = plan(before, vec![edit(0, 0, 4, "surface")]);
    let group = group(vec![reference(0, 0, 4)]);

    let mut gained = Scripted {
        answer: locations(&[(0, 0), (3, 4)]),
    };
    let error = check_group(&mut gained, &plan, &group).await.unwrap_err();
    let text = error.to_string();
    assert!(
        text.contains("Usages that newly refer to it:\n  a.src:4:5"),
        "{text}"
    );
}
