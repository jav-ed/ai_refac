use crate::common;

// Real Kotlin language server against tests/fixtures/kotlin/jvm_project. Every
// test starts a Gradle import (about 30 s), so they are ignored by default:
//   REFAC_KOTLIN_SERVER=<install dir> REFAC_KOTLIN_TESTS=all cargo test --test kotlin server:: -- --ignored

use refac::drivers::kotlin::server::{self, KotlinServer};
use serde_json::json;
use url::Url;

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn the_server_imports_the_project_and_answers_requests() {
    common::kotlin::require_slow_tests();
    let project = common::setup_fixture("kotlin/jvm_project");
    let root = project.path().canonicalize().unwrap();
    let install = server::locate().unwrap();
    let mut server = KotlinServer::start(&install, &root).await.unwrap();

    let helper = root.join("src/main/kotlin/com/example/util/Helper.kt");
    let uri = Url::from_file_path(&helper).unwrap().to_string();
    let symbols = server
        .request(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri } }),
        )
        .await
        .unwrap();
    assert!(
        symbols.to_string().contains("Helper"),
        "document symbols should name the class: {symbols}"
    );
    server.shutdown().await;
}

/// What `refac doctor kotlin` runs: `initialize` on an empty folder and a stop,
/// with no Gradle import. A server that changed what it offers fails here.
#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER)"]
async fn the_start_check_proves_the_server_starts_without_importing_a_project() {
    common::kotlin::require_slow_tests();
    let folder = std::env::var_os(server::SERVER_ENV).expect("REFAC_KOTLIN_SERVER is set");
    let executable = std::path::Path::new(&folder).join("bin/intellij-server");
    let began = std::time::Instant::now();
    server::check_start(&executable).await.unwrap();
    assert!(
        began.elapsed() < std::time::Duration::from_secs(30),
        "the start check imported a project: {:?}",
        began.elapsed()
    );
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_broken_gradle_build_fails_loudly_with_the_import_log() {
    common::kotlin::require_slow_tests();
    let project = common::setup_fixture("kotlin/jvm_project");
    let root = project.path().canonicalize().unwrap();
    std::fs::write(
        root.join("build.gradle.kts"),
        "this is not a build script\n",
    )
    .unwrap();
    let install = server::locate().unwrap();

    let error = match KotlinServer::start(&install, &root).await {
        Ok(_) => panic!("a broken build must not be reported as ready"),
        Err(error) => format!("{error:#}"),
    };
    assert!(error.contains("was not ready"), "{error}");
    assert!(
        error.contains("build.gradle.kts"),
        "the import log should name the broken script: {error}"
    );
}
