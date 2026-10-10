use crate::common;
use refac::drivers::kotlin::moves::move_files;

// The quick set: the only Kotlin tests that run by default, and together they
// have to fit into 30 seconds. One Kotlin server for jvm_project (17 to 18 s
// to start, 9 s when the Gradle daemon of an earlier run is still up), then one
// move of about 5 seconds, on the entry point the tool runs. Measured
// 2026-10-10: 22 to 23 s of test time with a cold Gradle daemon (24 to 25 s
// wall), 14 s with a warm one. A second test (the member rename) took the cold
// run to 28 s (29.5 s wall) and was dropped for the margin; the rename tests
// stay in `rename::`. Add a test here only after timing the whole set again,
// cold and warm. The first run after a reboot or in a fresh container reads
// the 1.2 GB server from a cold disk and takes about 42 s once.
// There is no compile check here (starting a Gradle daemon for it costs
// 12 s); the modules outside the set end every scenario with one, and run
// only when asked for (REFAC_KOTLIN_TESTS=all, see kotlin_Server.md).
// Run with: REFAC_KOTLIN_SERVER=<install dir> cargo test --test kotlin quick:: -- --ignored --test-threads=1

const K: &str = "src/main/kotlin/com/example";

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
