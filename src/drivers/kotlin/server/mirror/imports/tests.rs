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
    let kept = restore(BEFORE, after, &accounted, &|_| None);
    assert_eq!(
        kept,
        "package com.new\n\nimport androidx.compose.foundation.layout.padding\nimport androidx.compose.ui.Modifier\nimport com.new.Sibling\nimport com.util.Helper\n\nfun f() = Modifier.padding(1)\n"
    );
}

#[test]
fn an_import_of_the_package_a_file_left_may_go() {
    let after = BEFORE.replace("import com.old.Sibling\n", "");
    assert_eq!(restore(BEFORE, &after, &accounted, &|_| None), after);
}

#[test]
fn a_restored_import_keeps_its_place_even_at_the_top() {
    let before = "package a\n\nimport x.First\nimport x.Second\n\nclass C\n";
    let after = "package a\n\nimport x.Second\n\nclass C\n";
    assert_eq!(restore(before, after, &|_| false, &|_| None), before);
    // With no import left, the block goes back under the package line.
    let after = "package a\n\nclass C\n";
    assert_eq!(
        restore(before, after, &|_| false, &|_| None),
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
    // The moved file declares the class that the other file imports.
    let moved_text = format!("{BEFORE}\nclass Moved\n");
    let dir = project(&[
        ("src/Moved.kt", &moved_text),
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
        applied(&uri(&dir.path().join("new/Moved.kt")), &moved_text),
        moved_text.replace("package com.old", "package com.new")
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

#[test]
fn a_move_calls_for_the_imports_of_what_moved_and_not_for_the_rest_of_the_package() {
    let cause = Cause::Move {
        pairs: Vec::new(),
        left: BTreeMap::from([(
            "com.old".to_string(),
            BTreeMap::from([
                ("Moved".to_string(), Some("com.new".to_string())),
                ("helper".to_string(), Some("com.new".to_string())),
            ]),
        )]),
    };

    assert!(cause.accounts_for("com.old.Moved"));
    assert!(cause.accounts_for("com.old.Moved.Companion.make"));
    assert!(cause.accounts_for("com.old.helper"));
    assert!(cause.accounts_for("com.old.*"));
    // Something that stayed in the package, or another package.
    assert!(!cause.accounts_for("com.old.HairlineDivider"));
    assert!(!cause.accounts_for("com.older.Moved"));
    assert!(!cause.accounts_for("com.other.Moved"));
}

/// Found on a real project: the server dropped `import ...components.HairlineDivider`
/// from a file that only used moved classes (the import was unused) and
/// wrapped a call over four lines. A move rewrites the import of the class that
/// moved and nothing else.
#[test]
fn a_move_changes_the_import_of_what_moved_and_leaves_unused_imports_and_layout_alone() {
    let other = "package com.other\n\nimport com.old.HairlineDivider\nimport com.old.Moved\n\nfun g() {\n    Moved(1, 2, { go() })\n}\n";
    let dir = project(&[
        ("src/Moved.kt", "package com.old\n\nclass Moved\n"),
        ("src/Other.kt", other),
    ]);
    let (mut answer, request) = answer_for(dir.path(), "src/Moved.kt", json!([]));
    let other_uri = uri(&dir.path().join("src/Other.kt"));
    answer["changes"][&other_uri] = json!([
        { "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } }, "newText": "" },
        { "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 20 } }, "newText": "import com.new.Moved" },
        { "range": { "start": { "line": 6, "character": 0 }, "end": { "line": 6, "character": 25 } }, "newText": "    Moved(\n        1,\n        2,\n        { go() }\n    )" },
    ]);

    keep_imports(
        "workspace/willRenameFiles",
        &mut answer,
        &request,
        &|path| path.to_path_buf(),
    )
    .unwrap();

    let edits = serde_json::from_value(answer["changes"][&other_uri].clone()).unwrap();
    assert_eq!(
        apply_text_edits(other, edits).unwrap(),
        "package com.other\n\nimport com.old.HairlineDivider\nimport com.new.Moved\n\nfun g() {\n    Moved(1, 2, { go() })\n}\n"
    );
}

#[test]
fn an_unused_import_of_a_moved_class_that_the_server_deleted_is_written_for_the_new_package() {
    let before = "package com.other\n\nimport com.old.Moved\nimport x.Y\n\nclass C\n";
    let rewritten = |fqn: &str| (fqn == "com.old.Moved").then(|| "com.new.Moved".to_string());
    let accounted = |fqn: &str| fqn.starts_with("com.old.");

    // The server deleted it as unused.
    let dropped = "package com.other\n\nimport x.Y\n\nclass C\n";
    assert_eq!(
        restore(before, dropped, &accounted, &rewritten),
        "package com.other\n\nimport com.new.Moved\nimport x.Y\n\nclass C\n"
    );
    // The server rewrote it itself: nothing is added twice.
    let done = "package com.other\n\nimport com.new.Moved\nimport x.Y\n\nclass C\n";
    assert_eq!(restore(before, done, &accounted, &rewritten), done);
}

#[test]
fn a_moved_class_is_not_imported_into_the_package_it_moved_to_and_a_renamed_file_is_not_guessed() {
    let dir = project(&[
        (
            "src/main/kotlin/com/old/Moved.kt",
            "package com.old\n\nclass Moved\n",
        ),
        (
            "src/main/kotlin/com/old/Renamed.kt",
            "package com.old\n\nclass Renamed\n",
        ),
    ]);
    let root = dir.path().join("src/main/kotlin/com");
    let pair = |from: &str, to: &str| json!({ "oldUri": uri(&root.join(from)), "newUri": uri(&root.join(to)) });
    let request = json!({ "files": [
        pair("old/Moved.kt", "new/Moved.kt"),
        pair("old/Renamed.kt", "old/Other.kt"),
    ] });

    let cause = Cause::read_move(&request).unwrap();

    assert_eq!(
        cause.rewritten("com.old.Moved", Some("com.other")),
        Some("com.new.Moved".to_string())
    );
    assert_eq!(
        cause.rewritten("com.old.Moved.Companion.make", None),
        Some("com.new.Moved.Companion.make".to_string())
    );
    // The file that imports it is in the package it moved to.
    assert_eq!(cause.rewritten("com.old.Moved", Some("com.new")), None);
    // A rename of the file renames its class: the new name is the server's.
    assert!(cause.accounts_for("com.old.Renamed"));
    assert_eq!(cause.rewritten("com.old.Renamed", None), None);
    // Something that stayed.
    assert!(!cause.accounts_for("com.old.Stayed"));
    assert_eq!(cause.rewritten("com.old.Stayed", None), None);
}
