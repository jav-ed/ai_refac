use crate::common;
use crate::common::dry_run::{assert_plan_was_carried_out, plan_on_fresh_copy, tree_without};
use crate::common::project::Project;

// Real Kotlin language server against tests/fixtures/kotlin/kmp_project, a
// Kotlin Multiplatform build the server cannot import itself (its Gradle
// import prints "Failed to find 'target' in Kotlin extension" and no source
// set is known afterwards). refac gives the server a plain-JVM mirror of the
// source sets instead. The fixture:
//   commonMain  util/Helper.kt (expect fun platformName), app/Greeter.kt,
//               app/Screen.kt (Compose-style imports, delegate `by`),
//               other/Wild.kt (wildcard import, aliased import)
//   commonTest  app/GreeterCheck.kt      jvmMain  util/Platform.kt (actual), app/Main.kt
//   library/    stands for Compose; no source set, so the mirror does not hold it
// Every test ends with a Gradle compile of main and test: a move is right when
// the real project builds.
// Run with: REFAC_KOTLIN_SERVER=<install dir> REFAC_KOTLIN_TESTS=all cargo test --test kotlin multiplatform:: -- --ignored --test-threads=1

use refac::drivers::kotlin::moves::{MoveReport, move_files};
use refac::drivers::kotlin::rename::{RenameReport, RenameRequest, rename_symbol};
use std::path::Path;

const COMMON: &str = "src/commonMain/kotlin/com/example";
const JVM: &str = "src/jvmMain/kotlin/com/example";
const COMPILE: &[&str] = &["compileKotlinJvm", "compileTestKotlinJvm"];
const SCRATCH: &[&str] = &[".gradle", ".kotlin", ".idea", "build"];

/// For a test that is refused before a server starts and so needs none.
fn setup() -> tempfile::TempDir {
    common::kotlin::require_server();
    common::setup_fixture("kotlin/kmp_project")
}

/// The tests that use the server share one (common::pool).
async fn lease() -> common::pool::Lease {
    common::pool::lease("kotlin/kmp_project").await
}

fn pair(from: &str, to: &str) -> (String, String) {
    (from.to_string(), to.to_string())
}

async fn run(project: &Path, moves: &[(String, String)]) -> MoveReport {
    move_files(moves, Some(project))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"))
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn files_of_a_multiplatform_module_move_and_every_source_set_follows() {
    let project = lease().await;

    let report = run(
        project.path(),
        &[
            pair(
                &format!("{COMMON}/app/Greeter.kt"),
                &format!("{COMMON}/ui/Greeter.kt"),
            ),
            pair(
                &format!("{COMMON}/app/Screen.kt"),
                &format!("{COMMON}/ui/Screen.kt"),
            ),
        ],
    )
    .await;

    let read = |relative: &str| common::read_file(project.path(), relative);
    assert!(read(&format!("{COMMON}/ui/Greeter.kt")).starts_with("package com.example.ui"));
    assert!(!project.path().join(COMMON).join("app/Greeter.kt").exists());
    // commonMain, commonTest and jvmMain are all edited, and the wildcard
    // import that covered the old package becomes the import of the class.
    for user in [
        format!("{COMMON}/other/Wild.kt"),
        "src/commonTest/kotlin/com/example/app/GreeterCheck.kt".to_string(),
        format!("{JVM}/app/Main.kt"),
    ] {
        let text = read(&user);
        assert!(
            text.contains("import com.example.ui.Greeter"),
            "{user}:\n{text}"
        );
        assert!(!text.contains("com.example.app.Greeter"), "{user}:\n{text}");
    }
    assert!(
        read(&format!("{COMMON}/other/Wild.kt")).contains("import com.example.util.Helper as Aid")
    );
    assert!(
        report
            .notes
            .iter()
            .any(|note| note.contains("Kotlin Multiplatform")),
        "{:?}",
        report.notes
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn imports_that_only_a_library_the_server_cannot_see_explains_stay() {
    let project = lease().await;

    run(
        project.path(),
        &[pair(
            &format!("{COMMON}/app/Screen.kt"),
            &format!("{COMMON}/ui/Screen.kt"),
        )],
    )
    .await;

    let screen = common::read_file(project.path(), &format!("{COMMON}/ui/Screen.kt"));
    assert!(screen.starts_with("package com.example.ui"), "{screen}");
    // `by` reaches getValue and setValue through their imports alone: with the
    // library unknown, the server takes them for unused and deletes them.
    for kept in [
        "import androidx.compose.runtime.getValue",
        "import androidx.compose.runtime.setValue",
        "import androidx.compose.foundation.layout.padding",
    ] {
        assert!(screen.contains(kept), "{kept} is gone:\n{screen}");
    }
    // Greeter stayed behind, so the moved file now has to import it.
    assert!(
        screen.contains("import com.example.app.Greeter"),
        "{screen}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_file_or_symbol_that_is_expect_or_actual_is_refused_before_the_server_starts() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());

    // The mirror holds an expect and its actual as two declarations of one
    // name, which the server cannot move; refac says so instead of passing
    // its "would clash" through.
    for file in [
        format!("{COMMON}/util/Helper.kt"),
        format!("{JVM}/util/Platform.kt"),
    ] {
        let target = file.replace("/util/", "/base/");
        let error = move_files(&[pair(&file, &target)], Some(project.path()))
            .await
            .err()
            .unwrap_or_else(|| panic!("{file} should be refused"));
        let message = format!("{error:#}");
        assert!(message.contains("`expect` or `actual` (line"), "{message}");
        assert!(message.contains("by hand"), "{message}");
    }
    // The same for a symbol the named file declares expect.
    let error = rename(
        project.path(),
        &format!("{COMMON}/util/Helper.kt"),
        "platformName",
        "systemName",
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("an expect symbol should not be renamed"));
    let message = format!("{error:#}");
    assert!(
        message.contains("`platformName` is declared `expect`"),
        "{message}"
    );
    assert_eq!(before, common::kotlin::snapshot(project.path()));
}

async fn rename(
    project: &Path,
    file: &str,
    symbol: &str,
    new_name: &str,
) -> anyhow::Result<RenameReport> {
    rename_symbol(RenameRequest {
        project_path: project.to_path_buf(),
        file: file.into(),
        symbol: symbol.to_string(),
        new_name: new_name.to_string(),
        line: None,
        column: None,
        dry_run: false,
    })
    .await
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_symbol_rename_reaches_every_source_set() {
    let project = lease().await;
    let file = format!("{COMMON}/util/Helper.kt");

    let report = rename(project.path(), &file, "decorate", "embellish")
        .await
        .unwrap_or_else(|error| panic!("the rename failed: {error:#}"));

    // The declaration and a use in commonMain, a use in commonTest, a use in jvmMain.
    for user in [
        file,
        format!("{COMMON}/app/Greeter.kt"),
        "src/commonTest/kotlin/com/example/app/GreeterCheck.kt".to_string(),
        format!("{JVM}/app/Main.kt"),
    ] {
        let text = common::read_file(project.path(), &user);
        assert!(text.contains("embellish("), "{user}:\n{text}");
        assert!(!text.contains("decorate("), "{user}:\n{text}");
    }
    assert!(
        report
            .notes
            .iter()
            .any(|note| note.contains("Kotlin Multiplatform")),
        "{:?}",
        report.notes
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_rename_reaches_a_use_inside_a_call_of_a_library_function() {
    let project = lease().await;

    // Screen.kt calls `greet` in the lambda of `LaunchedEffect`, which the
    // mirror does not hold.
    rename(
        project.path(),
        &format!("{COMMON}/app/Greeter.kt"),
        "greet",
        "welcome",
    )
    .await
    .unwrap_or_else(|error| panic!("the rename failed: {error:#}"));

    let screen = common::read_file(project.path(), &format!("{COMMON}/app/Screen.kt"));
    assert!(screen.contains("welcome(\"screen"), "{screen}");
    // The imports the server cannot account for were not part of the edit.
    assert!(
        screen.contains("import androidx.compose.runtime.getValue"),
        "{screen}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
async fn a_failure_before_the_server_is_not_blamed_on_the_copy() {
    let project = common::setup_fixture("kotlin/kmp_project");

    let error = rename(
        project.path(),
        &format!("{COMMON}/app/Greeter.kt"),
        "nothing",
        "something",
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("a name that is not in the file should be refused"));

    let message = format!("{error:#}");
    assert!(
        message.contains("does not appear as an identifier"),
        "{message}"
    );
    assert!(!message.contains("Multiplatform"), "{message}");
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn the_plan_of_a_multiplatform_move_is_what_the_move_does() {
    let leased = lease().await;
    let project = Project::over(leased.path());
    let from = format!("{COMMON}/app/Greeter.kt");
    let to = format!("{COMMON}/ui/Greeter.kt");

    // The plan is made by the binary on a copy; the move runs on the shared server.
    let plan = plan_on_fresh_copy("kotlin/kmp_project", &[(&from, &to)], SCRATCH);
    let before = tree_without(&project, SCRATCH);
    move_files(&[(from, to)], Some(leased.path()))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"));
    assert_plan_was_carried_out(&project, &plan, &before, SCRATCH);

    // The moved file and the files of three source sets that import it.
    assert!(plan["edited_files"].as_u64().unwrap() >= 5, "{plan}");
    assert!(
        plan["notes"].to_string().contains("Kotlin Multiplatform"),
        "{plan}"
    );
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_move_to_another_source_set_is_refused_before_the_server_starts() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());

    let error = move_files(
        &[pair(
            &format!("{COMMON}/app/Greeter.kt"),
            &format!("{JVM}/app/Greeter.kt"),
        )],
        Some(project.path()),
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("a move between source sets should be refused"));

    assert!(
        format!("{error:#}").contains("different modules or source sets"),
        "{error:#}"
    );
    assert_eq!(before, common::kotlin::snapshot(project.path()));
}
