use crate::common;

// The same entry points the CLI and the MCP tool call, with Kotlin paths:
// `handle_refactor` routes .kt files and Kotlin directories to the Kotlin
// driver and shows its notes; `handle_rename` routes .kt files to the Kotlin
// rename. Run with: REFAC_KOTLIN_SERVER=<install dir> cargo test --test kotlin dispatch:: -- --ignored --test-threads=1
// The tests share one server (common::pool): it starts once for a fixture and each test
// gets the project as the fixture was; the entry points below use it for that project.

use refac::drivers::symbol::rename::RenameRequest;
use refac::logic::rename::handle_rename;
use refac::logic::{RefactorRequest, handle_refactor};

const K: &str = "src/main/kotlin/com/example";

fn move_request(project: &std::path::Path, from: &str, to: &str) -> RefactorRequest {
    RefactorRequest {
        source_path: vec![format!("{K}/{from}")],
        target_path: Some(vec![format!("{K}/{to}")]),
        operation: "move".to_string(),
        project_path: Some(project.to_string_lossy().into_owned()),
    }
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_kotlin_move_is_routed_and_its_notes_reach_the_response() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    let response = handle_refactor(move_request(
        project.path(),
        "app/Main.kt",
        "launch/Main.kt",
    ))
    .await
    .unwrap_or_else(|error| panic!("the move failed: {error:#}"));

    assert!(response.contains("Kotlin results"), "{response}");
    assert!(response.contains("// Note:"), "{response}");
    assert!(response.contains("build.gradle.kts"), "{response}");
    // One Kotlin file on its own: the response says what that cost and what
    // the batch form is.
    assert!(response.contains("This Kotlin move took"), "{response}");
    assert!(
        response.contains("--source-path a.kt --source-path b.kt"),
        "{response}"
    );
    assert!(
        common::read_file(project.path(), &format!("{K}/launch/Main.kt"))
            .starts_with("package com.example.launch")
    );
    project.assert_compiles(&["compileKotlin", "compileJava"]);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_directory_with_kotlin_sources_is_routed_to_the_kotlin_driver() {
    let project = common::pool::lease("kotlin/jvm_project").await;
    // A README that points at the folder and at a file in it follows the move.
    std::fs::write(
        project.path().join("README.md"),
        format!("The [helpers]({K}/util/) and [Helper]({K}/util/Helper.kt).\n"),
    )
    .unwrap();

    let response = handle_refactor(move_request(project.path(), "util", "common"))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"));

    assert!(response.contains("Kotlin results"), "{response}");
    // A folder is already many files in one server start: no advice to batch.
    assert!(!response.contains("This Kotlin move took"), "{response}");
    assert!(
        response.contains("Markdown links to the moved files"),
        "{response}"
    );
    assert!(
        common::read_file(project.path(), &format!("{K}/common/Helper.kt"))
            .starts_with("package com.example.common")
    );
    assert_eq!(
        common::read_file(project.path(), "README.md"),
        format!("The [helpers]({K}/common/) and [Helper]({K}/common/Helper.kt).\n")
    );
    project.assert_compiles(&["compileKotlin", "compileJava"]);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_kotlin_rename_is_routed_by_the_file_extension() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    let report = handle_rename(RenameRequest {
        project_path: project.path().to_path_buf(),
        file: format!("{K}/util/Helper.kt").into(),
        symbol: "shout".to_string(),
        new_name: "yell".to_string(),
        line: None,
        column: None,
        dry_run: false,
    })
    .await
    .unwrap_or_else(|error| panic!("the rename failed: {error:#}"));

    assert!(report.edits >= 2, "{report:?}");
    assert!(
        report
            .notes
            .iter()
            .any(|note| note.starts_with("This Kotlin rename took")),
        "{:?}",
        report.notes
    );
    assert!(common::read_file(project.path(), &format!("{K}/util/Helper.kt")).contains("yell"));
    project.assert_compiles(&["compileKotlin", "compileJava"]);
}

#[tokio::test]
async fn symbol_rename_names_both_supported_languages_when_it_refuses_a_file() {
    let error = handle_rename(RenameRequest {
        project_path: std::env::temp_dir(),
        file: "Thing.java".into(),
        symbol: "a".to_string(),
        new_name: "b".to_string(),
        line: None,
        column: None,
        dry_run: true,
    })
    .await
    .err()
    .unwrap()
    .to_string();
    assert!(error.contains("Kotlin"), "{error}");
    assert!(error.contains("TypeScript"), "{error}");
}
