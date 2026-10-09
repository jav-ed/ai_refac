use crate::common;

// Real Kotlin language server against tests/fixtures/kotlin/jvm_project
// (see kotlin_moves.rs for the layout) and android_project.
//   Helper.kt: line 5 class Helper, 6 var counter, 8 fun decorate(text), 14 fun
//   shout(text), 16 fun Helper.undecorate(text); three different `text` parameters.
// Run with: REFAC_KOTLIN_SERVER=<install dir> [ANDROID_HOME=<sdk>] \
//   cargo test --test kotlin rename:: -- --ignored

use refac::drivers::kotlin::rename::{
    RenameReport, RenameRequest, rename_all_symbols, rename_symbol,
};
use std::path::Path;

const K: &str = "src/main/kotlin/com/example";
const HELPER: &str = "src/main/kotlin/com/example/util/Helper.kt";
const COMPILE: &[&str] = &["compileKotlin", "compileJava"];

fn request(project: &Path, file: &str, symbol: &str, new_name: &str) -> RenameRequest {
    RenameRequest {
        project_path: project.to_path_buf(),
        file: file.into(),
        symbol: symbol.to_string(),
        new_name: new_name.to_string(),
        line: None,
        column: None,
        dry_run: false,
    }
}

async fn rename(request: RenameRequest) -> RenameReport {
    rename_symbol(request)
        .await
        .unwrap_or_else(|error| panic!("the rename failed: {error:#}"))
}

async fn refused(request: RenameRequest) -> String {
    match rename_symbol(request).await {
        Ok(report) => panic!(
            "the rename should be refused, but changed {:?}",
            report.files
        ),
        Err(error) => format!("{error:#}"),
    }
}

fn setup() -> tempfile::TempDir {
    common::kotlin::require_server();
    common::setup_fixture("kotlin/jvm_project")
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_member_function_is_renamed_in_kotlin_and_java_callers() {
    let project = setup();

    let report = rename(request(project.path(), HELPER, "decorate", "embellish")).await;

    assert_eq!(report.files.len(), 3, "{:?}", report.files);
    assert!(common::read_file(project.path(), HELPER).contains("fun embellish(text: String)"));
    let greeter = common::read_file(project.path(), &format!("{K}/app/Greeter.kt"));
    assert!(greeter.contains("helper.embellish("), "{greeter}");
    let java = common::read_file(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
    );
    assert!(java.contains("helper.embellish("), "{java}");
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_property_is_renamed_with_its_java_accessor() {
    let project = setup();

    rename(request(project.path(), HELPER, "counter", "callCount")).await;

    let main = common::read_file(project.path(), &format!("{K}/app/Main.kt"));
    assert!(main.contains("helper.callCount"), "{main}");
    let java = common::read_file(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
    );
    assert!(java.contains("getCallCount()"), "{java}");
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_class_is_renamed_together_with_its_file() {
    let project = setup();

    let report = rename(request(
        project.path(),
        &format!("{K}/app/Greeter.kt"),
        "Greeter",
        "Welcomer",
    ))
    .await;

    assert!(!project.path().join(K).join("app/Greeter.kt").exists());
    assert!(
        common::read_file(project.path(), &format!("{K}/app/Welcomer.kt"))
            .contains("class Welcomer")
    );
    assert!(
        common::read_file(project.path(), &format!("{K}/app/Main.kt")).contains("Welcomer(helper)")
    );
    assert!(
        report.notes.iter().any(|note| note.contains("Welcomer.kt")),
        "{:?}",
        report.notes
    );
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

/// Greeter.kt becomes Welcomer.kt with its class; the second rename names the
/// file by its new path, so the server must have been told about the move. The
/// third runs in the file the first one did not touch.
#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_batch_follows_a_class_file_that_an_earlier_rename_moved() {
    let project = setup();
    let greeter = format!("{K}/app/Greeter.kt");
    let welcomer = format!("{K}/app/Welcomer.kt");

    let reports = rename_all_symbols(vec![
        request(project.path(), &greeter, "Greeter", "Welcomer"),
        request(project.path(), &welcomer, "greet", "welcome"),
        request(project.path(), HELPER, "decorate", "embellish"),
    ])
    .await
    .unwrap_or_else(|error| panic!("the batch failed: {error:#}"));

    assert_eq!(reports.len(), 3);
    assert!(!project.path().join(&greeter).exists());
    let moved = common::read_file(project.path(), &welcomer);
    assert!(moved.contains("class Welcomer"), "{moved}");
    assert!(moved.contains("fun welcome(name: String)"), "{moved}");
    assert!(moved.contains("helper.embellish("), "{moved}");
    let main = common::read_file(project.path(), &format!("{K}/app/Main.kt"));
    assert!(main.contains("Welcomer(helper)"), "{main}");
    assert!(main.contains("greeter.welcome(\"world\")"), "{main}");
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

/// A dry run of several renames carries the batch out on a copy of the project
/// (a class rename moves its file and edits Android XML, which no in-memory
/// view follows): the plan names the files the batch then edits, a rename that
/// depends on the one before it works, and the project is not touched.
#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_dry_run_of_a_batch_plans_what_the_batch_does() {
    let greeter = format!("{K}/app/Greeter.kt");
    let welcomer = format!("{K}/app/Welcomer.kt");
    let batch = |project: &Path, dry_run: bool| {
        let mut requests = vec![
            request(project, &greeter, "Greeter", "Welcomer"),
            request(project, &welcomer, "greet", "welcome"),
            request(project, HELPER, "decorate", "embellish"),
        ];
        for request in &mut requests {
            request.dry_run = dry_run;
        }
        requests
    };

    let planned = setup();
    let before = common::kotlin::snapshot(planned.path());
    let plan = rename_all_symbols(batch(planned.path(), true))
        .await
        .unwrap_or_else(|error| panic!("the dry run failed: {error:#}"));
    assert_eq!(common::kotlin::snapshot(planned.path()), before);
    assert!(plan.iter().all(|report| report.dry_run));

    let carried_out = setup();
    let done = rename_all_symbols(batch(carried_out.path(), false))
        .await
        .unwrap_or_else(|error| panic!("the batch failed: {error:#}"));

    assert_eq!(plan.len(), done.len());
    for (index, (planned, done)) in plan.iter().zip(&done).enumerate() {
        assert_eq!(planned.files, done.files, "rename {}", index + 1);
        assert_eq!(planned.edits, done.edits, "rename {}", index + 1);
    }
    // No note names the folder of the copy.
    for note in plan.iter().flat_map(|report| &report.notes) {
        assert!(!note.contains("refac-dry-run-"), "{note}");
    }
}

/// The failing rename comes after one that moved a file: the move is undone
/// as well as the edits.
#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_failing_batch_gives_back_the_moved_file_and_every_edit() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());

    let error = rename_all_symbols(vec![
        request(
            project.path(),
            &format!("{K}/app/Greeter.kt"),
            "Greeter",
            "Welcomer",
        ),
        request(project.path(), HELPER, "no_such_symbol", "something_else"),
    ])
    .await
    .expect_err("the second rename cannot work");

    let message = format!("{error:#}");
    assert!(message.contains("Rename 2 of 2"), "{message}");
    assert!(
        message.contains("1 earlier rename(s) were undone"),
        "{message}"
    );
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_dry_run_plans_and_verifies_but_writes_nothing() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());
    let mut dry = request(project.path(), HELPER, "decorate", "embellish");
    dry.dry_run = true;

    let report = rename(dry).await;

    assert!(report.dry_run);
    assert_eq!(report.files.len(), 3, "{:?}", report.files);
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_name_shared_by_several_symbols_is_listed_and_chosen_by_line() {
    let project = setup();

    let message = refused(request(project.path(), HELPER, "text", "value")).await;
    assert!(message.contains("several different symbols"), "{message}");
    assert!(message.contains("14:"), "{message}");

    let mut chosen = request(project.path(), HELPER, "text", "value");
    chosen.line = Some(14);
    let report = rename(chosen).await;
    assert_eq!(report.files.len(), 1, "{:?}", report.files);
    let helper = common::read_file(project.path(), HELPER);
    assert!(
        helper.contains("fun shout(value: String): String = value.uppercase()"),
        "{helper}"
    );
    assert!(helper.contains("fun decorate(text: String)"), "{helper}");
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_rename_that_shadows_another_declaration_is_refused_and_changes_nothing() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());

    // A member `decorate` wins over an extension `decorate`: callers of the
    // extension would silently call the member.
    let message = refused(request(project.path(), HELPER, "undecorate", "decorate")).await;

    assert!(message.contains("not faithful"), "{message}");
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_clash_with_a_member_in_the_same_class_is_refused() {
    let project = setup();
    let before = common::kotlin::snapshot(project.path());

    let message = refused(request(project.path(), HELPER, "counter", "prefix")).await;

    assert!(!message.is_empty());
    assert_eq!(common::kotlin::snapshot(project.path()), before);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn text_that_is_not_a_symbol_is_refused() {
    let project = setup();

    // "hello" only appears inside a string literal.
    let message = refused(request(project.path(), HELPER, "hello", "goodbye")).await;

    assert!(message.contains("Cannot rename"), "{message}");
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn an_android_class_rename_updates_the_layout_tag() {
    common::kotlin::require_server();
    common::kotlin::require_android_sdk();
    let project = common::setup_fixture("kotlin/android_project");
    let view = "app/src/main/java/com/example/droid/widgets/BadgeView.kt";

    let report = rename(request(project.path(), view, "BadgeView", "CounterView")).await;

    let layout = common::read_file(project.path(), "app/src/main/res/layout/activity_main.xml");
    assert!(
        layout.contains("<com.example.droid.widgets.CounterView"),
        "{layout}"
    );
    assert!(
        report
            .files
            .iter()
            .any(|(path, _)| path.ends_with("activity_main.xml")),
        "{:?}",
        report.files
    );
    common::kotlin::assert_compiles(
        project.path(),
        &[":app:compileDebugKotlin", ":app:compileDebugJavaWithJavac"],
    );
}

// Refusals that come before the server starts run without it.
#[tokio::test]
async fn bad_requests_fail_before_any_server_starts() {
    let project = common::setup_fixture("kotlin/jvm_project");

    let keyword = refused(request(project.path(), HELPER, "decorate", "class")).await;
    assert!(keyword.contains("keyword"), "{keyword}");
    let same = refused(request(project.path(), HELPER, "decorate", "decorate")).await;
    assert!(same.contains("equals the current name"), "{same}");
    let java = refused(request(
        project.path(),
        "src/main/java/com/example/legacy/JavaCaller.java",
        "run",
        "go",
    ))
    .await;
    assert!(java.contains(".kt"), "{java}");
    let missing = refused(request(project.path(), HELPER, "nothingHere", "something")).await;
    assert!(missing.contains("does not appear"), "{missing}");
}
