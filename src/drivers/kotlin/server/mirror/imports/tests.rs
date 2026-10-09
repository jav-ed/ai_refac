use super::super::tests::{project, uri};
use super::*;
use serde_json::json;

const BEFORE: &str = "package com.old\n\nimport androidx.compose.foundation.layout.padding\nimport androidx.compose.ui.Modifier\nimport com.old.Sibling\nimport com.util.Helper\n\nfun f() = Modifier.padding(1)\n";

fn accounted(fqn: &str) -> bool {
    fqn.starts_with("com.old.")
}

#[test]
fn an_import_the_server_dropped_for_a_missing_library_comes_back() {
    // The server saw `padding` as unused and deleted it, and rewrote the
    // import of the moved class.
    let after = "package com.new\n\nimport androidx.compose.ui.Modifier\nimport com.new.Sibling\nimport com.util.Helper\n\nfun f() = Modifier.padding(1)\n";
    let kept = restore(BEFORE, after, &accounted);
    assert_eq!(
        kept,
        "package com.new\n\nimport androidx.compose.foundation.layout.padding\nimport androidx.compose.ui.Modifier\nimport com.new.Sibling\nimport com.util.Helper\n\nfun f() = Modifier.padding(1)\n"
    );
}

#[test]
fn an_import_of_the_package_a_file_left_may_go() {
    let after = BEFORE.replace("import com.old.Sibling\n", "");
    assert_eq!(restore(BEFORE, &after, &accounted), after);
}

#[test]
fn a_restored_import_keeps_its_place_even_at_the_top() {
    let before = "package a\n\nimport x.First\nimport x.Second\n\nclass C\n";
    let after = "package a\n\nimport x.Second\n\nclass C\n";
    assert_eq!(restore(before, after, &|_| false), before);
    // With no import left, the block goes back under the package line.
    let after = "package a\n\nclass C\n";
    assert_eq!(
        restore(before, after, &|_| false),
        "package a\nimport x.First\nimport x.Second\n\nclass C\n"
    );
}

#[test]
fn edits_between_are_one_per_changed_run_of_tokens() {
    let before = "a\nb\nc\nd\ne\n";
    let after = "a\nB\nc\nd\ne\nf\n";
    let edits = edits_between(before, after);
    assert_eq!(edits.len(), 2);
    assert_eq!(
        lsp_types::Range::new(
            lsp_types::Position::new(1, 0),
            lsp_types::Position::new(1, 1)
        ),
        edits[0].range
    );
    assert_eq!(edits[0].new_text, "B");
    assert_eq!(edits[1].new_text, "f\n");
    assert!(edits_between(before, before).is_empty());
    // A changed name is just the name.
    let edits = edits_between("import a.b.Old\n", "import a.c.Old\n");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "c");
}

fn answer_for(dir: &Path, file: &str, after_edits: Value) -> (Value, Value) {
    let old = dir.join(file);
    let new = dir.join("new").join(old.file_name().unwrap());
    let request = json!({ "files": [{ "oldUri": uri(&old), "newUri": uri(&new) }] });
    // The server addresses the file under its new name.
    (json!({ "changes": { uri(&new): after_edits } }), request)
}

#[test]
fn the_answer_of_a_move_keeps_imports_and_drops_edits_that_only_tidy() {
    let dir = project(&[
        ("src/Moved.kt", BEFORE),
        (
            "src/Other.kt",
            "package com.other\n\nimport androidx.compose.runtime.Composable\nimport com.old.Moved\n",
        ),
    ]);
    let (mut answer, request) = answer_for(
        dir.path(),
        "src/Moved.kt",
        // Delete the `padding` import and rewrite the package line.
        json!([
            { "range": { "start": { "line": 0, "character": 8 }, "end": { "line": 0, "character": 15 } }, "newText": "com.new" },
            { "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } }, "newText": "" },
        ]),
    );
    // And an edit of another file that only deletes an unrelated import.
    let other = dir.path().join("src/Other.kt");
    answer["changes"][uri(&other)] = json!([
        { "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } }, "newText": "" },
        { "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 20 } }, "newText": "import com.new.Moved" },
    ]);

    keep_imports(
        "workspace/willRenameFiles",
        &mut answer,
        &request,
        &|path| path.to_path_buf(),
    )
    .unwrap();

    let changes = answer["changes"].as_object().unwrap();
    let applied = |key: &str, text: &str| {
        apply_text_edits(text, serde_json::from_value(changes[key].clone()).unwrap()).unwrap()
    };
    // The moved file takes its new package and keeps the import the server
    // deleted as unused.
    assert_eq!(
        applied(&uri(&dir.path().join("new/Moved.kt")), BEFORE),
        BEFORE.replace("package com.old", "package com.new")
    );
    // The other file takes the new import of the moved class and keeps its own.
    assert_eq!(
        applied(
            &uri(&other),
            "package com.other\n\nimport androidx.compose.runtime.Composable\nimport com.old.Moved\n"
        ),
        "package com.other\n\nimport androidx.compose.runtime.Composable\nimport com.new.Moved\n"
    );
}

#[test]
fn an_answer_that_changes_nothing_after_the_restore_is_empty() {
    let dir = project(&[("src/Moved.kt", BEFORE)]);
    let (mut answer, request) = answer_for(
        dir.path(),
        "src/Moved.kt",
        json!([{ "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } }, "newText": "" }]),
    );
    keep_imports(
        "workspace/willRenameFiles",
        &mut answer,
        &request,
        &|path| path.to_path_buf(),
    )
    .unwrap();
    assert!(
        answer["changes"].as_object().unwrap().is_empty(),
        "{answer}"
    );
}

#[test]
fn a_rename_keeps_what_it_does_not_call_for_and_lets_the_symbols_own_import_go() {
    let text = "package p\n\nimport androidx.compose.runtime.getValue\nimport x.Greeter\n\nval g = Greeter()\n";
    let dir = project(&[("src/A.kt", text)]);
    let file = dir.path().join("src/A.kt");
    // The cursor is on `Greeter` in the last line. The server renames it,
    // rewrites the import and, as a tidy-up, deletes the delegate import.
    let request = json!({
        "textDocument": { "uri": uri(&file) },
        "position": { "line": 5, "character": 10 },
        "newName": "Welcomer",
    });
    let mut answer = json!({ "changes": { uri(&file): [
        { "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 4, "character": 0 } },
          "newText": "import x.Welcomer\n" },
        { "range": { "start": { "line": 5, "character": 8 }, "end": { "line": 5, "character": 15 } },
          "newText": "Welcomer" },
    ] } });

    keep_imports("textDocument/rename", &mut answer, &request, &|path| {
        path.to_path_buf()
    })
    .unwrap();

    let after = apply_text_edits(
        text,
        serde_json::from_value(answer["changes"][uri(&file)].clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        after,
        "package p\n\nimport androidx.compose.runtime.getValue\nimport x.Welcomer\n\nval g = Welcomer()\n"
    );
}

#[test]
fn other_requests_are_not_touched() {
    let mut answer = json!({ "changes": { "file:///nowhere/A.kt": [] } });
    let before = answer.clone();
    keep_imports(
        "textDocument/references",
        &mut answer,
        &json!({}),
        &|path| path.to_path_buf(),
    )
    .unwrap();
    assert_eq!(answer, before);
}

#[test]
fn an_edit_on_a_line_whose_modifier_the_mirror_hides_is_refused() {
    // The mirror shows `expect` as spaces; the real file has the word.
    let mirrored = project(&[("src/A.kt", "package a\n\n       fun f(): String\n")]);
    let real = project(&[("src/A.kt", "package a\n\nexpect fun f(): String\n")]);
    let file = mirrored.path().join("src/A.kt");
    let request = json!({ "files": [{ "oldUri": uri(&file), "newUri": uri(&file) }] });
    let real_file = real.path().join("src/A.kt");
    let mut answer = json!({ "changes": { uri(&file): [
        { "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } },
          "newText": "       fun g(): String\n" },
    ] } });

    let error = keep_imports("workspace/willRenameFiles", &mut answer, &request, &|_| {
        real_file.clone()
    })
    .unwrap_err()
    .to_string();

    assert!(error.contains("expect"), "{error}");
    assert!(error.contains("line 3"), "{error}");
}
