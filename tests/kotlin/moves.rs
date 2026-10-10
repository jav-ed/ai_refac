use crate::common;

// Real Kotlin language server against tests/fixtures/kotlin/jvm_project:
//   util/Helper.kt   package com.example.util   (class Helper, shout, DEFAULT_GREETING)
//   app/Main.kt      same package as Greeter, so it imports it only after a move
//   app/Greeter.kt, cli/Runner.kt, and a Java caller in java/.../legacy.
// Every test ends with a Gradle compile: a move is right when the project builds.
// Run with: REFAC_KOTLIN_SERVER=<install dir> REFAC_KOTLIN_TESTS=all cargo test --test kotlin moves:: -- --ignored --test-threads=1
// The tests share one server (common::pool): it starts once for a fixture and each test
// gets the project as the fixture was; the entry points below use it for that project.

use refac::drivers::kotlin::moves::{MoveReport, move_files};
use std::path::Path;

const K: &str = "src/main/kotlin/com/example";
const COMPILE: &[&str] = &["compileKotlin", "compileJava"];

fn pair(from: &str, to: &str) -> (String, String) {
    (format!("{K}/{from}"), format!("{K}/{to}"))
}

async fn run(project: &Path, moves: &[(String, String)]) -> MoveReport {
    move_files(moves, Some(project))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"))
}

fn text(project: &Path, relative: &str) -> String {
    common::read_file(project, &format!("{K}/{relative}"))
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_file_moves_to_a_new_package_and_every_reference_follows() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    let report = run(
        project.path(),
        &[pair("util/Helper.kt", "common/Helper.kt")],
    )
    .await;

    assert!(!project.path().join(K).join("util/Helper.kt").exists());
    assert!(text(project.path(), "common/Helper.kt").starts_with("package com.example.common"));
    assert!(text(project.path(), "app/Main.kt").contains("import com.example.common.Helper"));
    assert!(
        text(project.path(), "app/Greeter.kt")
            .contains("import com.example.common.DEFAULT_GREETING")
    );
    assert!(text(project.path(), "cli/Runner.kt").contains("import com.example.common.undecorate"));
    let java = common::read_file(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
    );
    assert!(java.contains("import com.example.common.Helper;"), "{java}");
    assert!(report.edited.len() >= 4, "{:?}", report.edited);
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn files_for_different_directories_are_moved_in_one_session() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(
        project.path(),
        &[
            pair("util/Helper.kt", "common/Helper.kt"),
            pair("app/Greeter.kt", "greet/Greeter.kt"),
            pair("cli/Runner.kt", "launch/Runner.kt"),
        ],
    )
    .await;

    let runner = text(project.path(), "launch/Runner.kt");
    assert!(runner.starts_with("package com.example.launch"), "{runner}");
    assert!(
        runner.contains("import com.example.greet.Greeter"),
        "{runner}"
    );
    assert!(
        runner.contains("import com.example.common.Helper"),
        "{runner}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_package_directory_is_renamed_with_its_files() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(project.path(), &[pair("util", "common")]).await;

    assert!(!project.path().join(K).join("util").exists());
    assert!(text(project.path(), "common/Helper.kt").starts_with("package com.example.common"));
    assert!(text(project.path(), "app/Main.kt").contains("import com.example.common.shout"));
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_file_is_renamed_in_place_and_its_class_follows() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(project.path(), &[pair("app/Greeter.kt", "app/Welcomer.kt")]).await;

    assert!(text(project.path(), "app/Welcomer.kt").contains("class Welcomer"));
    assert!(text(project.path(), "app/Main.kt").contains("Welcomer(helper)"));
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_move_with_a_new_name_runs_as_move_then_rename() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    let report = run(
        project.path(),
        &[pair("app/Greeter.kt", "greet/Welcomer.kt")],
    )
    .await;

    let moved = text(project.path(), "greet/Welcomer.kt");
    assert!(moved.starts_with("package com.example.greet"), "{moved}");
    assert!(moved.contains("class Welcomer"), "{moved}");
    assert!(text(project.path(), "cli/Runner.kt").contains("import com.example.greet.Welcomer"));
    assert!(
        report.notes.iter().any(|note| note.contains("two steps")),
        "{:?}",
        report.notes
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_failure_in_a_later_group_restores_the_first_group() {
    let project = common::pool::lease("kotlin/jvm_project").await;
    // A file where the second target directory has to be created.
    std::fs::write(project.path().join(K).join("blocker"), "not a directory").unwrap();
    let before = common::kotlin::snapshot(project.path());

    let error = move_files(
        &[
            pair("util/Helper.kt", "common/Helper.kt"),
            pair("app/Greeter.kt", "blocker/Greeter.kt"),
        ],
        Some(project.path()),
    )
    .await
    .unwrap_err();

    assert!(format!("{error:#}").contains("undone"), "{error:#}");
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

// Refusals are decided before the server starts, so this one runs without it.
#[tokio::test]
async fn a_directory_of_java_sources_is_refused_before_anything_starts() {
    let project = common::setup_fixture("kotlin/jvm_project");
    let java = |name: &str| format!("src/main/java/com/example/{name}");
    let before = common::kotlin::snapshot(project.path());

    let error = move_files(&[(java("legacy"), java("bridge"))], Some(project.path()))
        .await
        .unwrap_err();

    assert!(
        error.to_string().contains("contains Java sources"),
        "{error:#}"
    );
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_kotlin_file_moves_between_the_kotlin_and_java_folders_of_one_source_set() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(
        project.path(),
        &[(
            format!("{K}/util/Helper.kt"),
            "src/main/java/com/example/common/Helper.kt".to_string(),
        )],
    )
    .await;

    let moved = common::read_file(project.path(), "src/main/java/com/example/common/Helper.kt");
    assert!(moved.starts_with("package com.example.common"), "{moved}");
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_build_script_that_names_the_old_main_class_is_reported() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    let report = run(project.path(), &[pair("app/Main.kt", "launch/Main.kt")]).await;

    // build.gradle.kts says mainClass.set("com.example.app.MainKt"): refac does
    // not edit build scripts, so it must say so.
    let note = report
        .notes
        .iter()
        .find(|note| note.contains("build.gradle.kts"))
        .unwrap_or_else(|| panic!("no note about the build script: {:?}", report.notes));
    assert!(note.contains("com.example.app.MainKt"), "{note}");
    assert!(note.contains("com.example.launch.MainKt"), "{note}");
    project.assert_compiles(COMPILE);
}
