//! The wording and the verdict of a move's answer: complete, partly done, or
//! not done at all.

use super::*;
use std::path::Path;

fn pair(source: &str, target: &str) -> (String, String) {
    (source.to_string(), target.to_string())
}

fn outcome(root: Option<&Path>) -> MoveOutcome<'_> {
    MoveOutcome {
        root,
        moved: BTreeMap::new(),
        notes: HashMap::new(),
        link_notes: None,
        failed: Vec::new(),
        skipped: Vec::new(),
        typescript_source_count: 0,
    }
}

fn failed(lang: &str, files: Pairs, error: &str) -> FailedGroup {
    FailedGroup {
        lang: lang.to_string(),
        files,
        error: error.to_string(),
    }
}

#[test]
fn a_move_that_worked_reads_as_success_and_shows_paths_below_the_root() {
    let root = Path::new("/project");
    let mut done = outcome(Some(root));
    done.moved.insert(
        "python".to_string(),
        vec![pair("/project/a.py", "/project/pkg/a.py")],
    );
    done.notes
        .insert("python".to_string(), vec!["look at strings".to_string()]);

    let text = done.render();

    assert!(done.is_complete());
    assert!(
        text.starts_with("// Alhamdulillah 1 requested path was successfully refactored:"),
        "{text}"
    );
    assert!(text.contains("// Python results:"), "{text}");
    assert!(text.contains("a.py -> pkg/a.py"), "{text}");
    assert!(!text.contains("/project/"), "{text}");
    assert!(text.contains("// Note: look at strings"), "{text}");
}

#[test]
fn a_failed_group_makes_the_move_partly_done_and_names_the_group() {
    let mut partly = outcome(None);
    partly
        .moved
        .insert("go".to_string(), vec![pair("a.go", "x/a.go")]);
    partly.failed.push(failed(
        "python",
        vec![pair("b.py", "x/b.py"), pair("c.py", "x/c.py")],
        "Rope refused",
    ));

    let text = partly.render();

    assert!(!partly.is_complete());
    assert!(
        text.starts_with(
            "// Partly done: 1 requested path moved, 2 failed. The groups that moved stay moved;"
        ),
        "{text}"
    );
    assert!(text.contains("// Failed:"), "{text}");
    assert!(text.contains("// Python — Rope refused"), "{text}");
    assert!(text.contains("c.py -> x/c.py"), "{text}");
}

#[test]
fn nothing_moved_is_not_a_success_even_without_a_failure() {
    let mut nothing = outcome(None);
    nothing.skipped.push("notes.xyz".to_string());

    let text = nothing.render();

    assert!(!nothing.is_complete());
    assert!(text.starts_with("// Nothing was moved."), "{text}");
    assert!(
        text.contains("// Skipped (unsupported extension):"),
        "{text}"
    );
    assert!(text.contains("notes.xyz"), "{text}");
}

#[test]
fn a_skipped_file_next_to_a_moved_one_stays_a_success_with_the_skip_listed() {
    let mut mixed = outcome(None);
    mixed
        .moved
        .insert("go".to_string(), vec![pair("a.go", "x/a.go")]);
    mixed.skipped.push("notes.xyz".to_string());

    let text = mixed.render();

    assert!(mixed.is_complete());
    assert!(text.contains("notes.xyz"), "{text}");
}
