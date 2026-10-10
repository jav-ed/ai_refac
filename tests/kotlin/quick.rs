use crate::common;
use refac::drivers::kotlin::moves::move_files;
use refac::drivers::kotlin::rename::{RenameRequest, rename_symbol};

// The quick set: the only Kotlin tests that run by default, and together they
// have to fit into 30 seconds. One Kotlin server for jvm_project (20 to 23 s
// to start, 10 s when the Gradle daemon of an earlier run is still up), then
// two operations of 3 to 4 seconds each, on the entry points the tool runs.
// Measured 2026-10-10 (two tests): 25 to 28 s of test time with a cold Gradle
// daemon (29.5 s wall in the last run, so a slow start of 22 s or more can
// pass 30 s), 17 s with a warm one. A third test did not fit (the cold run took
// 29 to 33 s), so the rollback test stays in `moves::`; add a test here only
// after timing the whole set again, cold and warm.
// There is no compile check here (starting a Gradle daemon for it costs
// 12 s); the modules outside the set end every scenario with one, and run
// only when asked for (REFAC_KOTLIN_TESTS=all, see kotlin_Server.md).
// Run with: REFAC_KOTLIN_SERVER=<install dir> cargo test --test kotlin quick:: -- --ignored --test-threads=1

const K: &str = "src/main/kotlin/com/example";
const HELPER: &str = "src/main/kotlin/com/example/util/Helper.kt";

fn pair(from: &str, to: &str) -> (String, String) {
    (format!("{K}/{from}"), format!("{K}/{to}"))
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_file_moves_and_every_reference_follows() {
    let project = common::pool::lease_quick("kotlin/jvm_project").await;

    let report = move_files(
        &[pair("util/Helper.kt", "common/Helper.kt")],
        Some(project.path()),
    )
    .await
    .unwrap_or_else(|error| panic!("the move failed: {error:#}"));

    let read = |relative: &str| common::read_file(project.path(), &format!("{K}/{relative}"));
    assert!(!project.path().join(K).join("util/Helper.kt").exists());
    assert!(read("common/Helper.kt").starts_with("package com.example.common"));
    assert!(read("app/Main.kt").contains("import com.example.common.Helper"));
    let java = common::read_file(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
    );
    assert!(java.contains("import com.example.common.Helper;"), "{java}");
    assert!(report.edited.len() >= 4, "{:?}", report.edited);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_member_is_renamed_in_kotlin_and_java_callers() {
    let project = common::pool::lease_quick("kotlin/jvm_project").await;

    let report = rename_symbol(RenameRequest {
        project_path: project.path().to_path_buf(),
        file: HELPER.into(),
        symbol: "decorate".to_string(),
        new_name: "embellish".to_string(),
        line: None,
        column: None,
        dry_run: false,
    })
    .await
    .unwrap_or_else(|error| panic!("the rename failed: {error:#}"));

    assert_eq!(report.files.len(), 3, "{:?}", report.files);
    assert!(common::read_file(project.path(), HELPER).contains("fun embellish(text: String)"));
    let java = common::read_file(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
    );
    assert!(java.contains("helper.embellish("), "{java}");
}
