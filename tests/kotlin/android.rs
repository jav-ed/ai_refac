use crate::common;

// Real Kotlin language server against tests/fixtures/kotlin/android_project
// (namespace com.example.droid, sources in src/main/java):
//   MainActivity.kt        in the namespace package, uses R, named by manifest and layout
//   widgets/BadgeView.kt   custom view used as a layout tag and from Launcher.java
//   sync/SyncService.kt    named by its full name in the manifest
//   ui/HomeFragment.kt     named by the navigation graph
// The server never edits XML and drops the implicit R when a file leaves the
// namespace package; these tests prove refac repairs both, judged by a compile.
// Run with: REFAC_KOTLIN_SERVER=<install dir> ANDROID_HOME=<sdk> \
//   cargo test --test kotlin android:: -- --ignored --test-threads=1
// The tests share one server (common::pool): it starts once for a fixture and each test
// gets the project as the fixture was; the entry points below use it for that project.

use refac::drivers::kotlin::moves::{MoveReport, move_files};
use refac::drivers::kotlin::rename::{RenameReport, RenameRequest, rename_symbol};
use std::path::Path;

const J: &str = "src/main/java/com/example/droid";
const RES: &str = "app/src/main/res";
const COMPILE: &[&str] = &[":app:compileDebugKotlin", ":app:compileDebugJavaWithJavac"];

fn app(path: &str) -> String {
    format!("app/{J}/{path}")
}

async fn run(project: &Path, moves: &[(String, String)]) -> MoveReport {
    move_files(moves, Some(project))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"))
}

async fn setup() -> common::pool::Lease {
    common::kotlin::require_android_sdk();
    common::pool::lease("kotlin/android_project").await
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn an_activity_moves_and_manifest_layout_and_r_follow() {
    let project = setup().await;

    let report = run(
        project.path(),
        &[(app("MainActivity.kt"), app("ui/MainActivity.kt"))],
    )
    .await;

    let manifest = common::read_file(project.path(), "app/src/main/AndroidManifest.xml");
    assert!(
        manifest.contains(r#"android:name=".ui.MainActivity""#),
        "{manifest}"
    );
    let layout = common::read_file(project.path(), &format!("{RES}/layout/activity_main.xml"));
    assert!(
        layout.contains(r#"tools:context=".ui.MainActivity""#),
        "{layout}"
    );
    let moved = common::read_file(project.path(), &app("ui/MainActivity.kt"));
    assert!(moved.contains("import com.example.droid.R\n"), "{moved}");
    assert!(
        report
            .edited
            .iter()
            .any(|path| path.ends_with("AndroidManifest.xml"))
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_custom_view_directory_is_renamed_and_its_layout_tag_follows() {
    let project = setup().await;

    run(project.path(), &[(app("widgets"), app("views"))]).await;

    let layout = common::read_file(project.path(), &format!("{RES}/layout/activity_main.xml"));
    assert!(
        layout.contains("<com.example.droid.views.BadgeView"),
        "{layout}"
    );
    let java = common::read_file(project.path(), &app("legacy/Launcher.java"));
    assert!(
        java.contains("import com.example.droid.views.BadgeView;"),
        "{java}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_service_and_a_fragment_are_renamed_in_manifest_and_navigation_graph() {
    let project = setup().await;

    run(
        project.path(),
        &[
            (app("sync/SyncService.kt"), app("work/SyncService.kt")),
            (app("ui/HomeFragment.kt"), app("home/HomeFragment.kt")),
        ],
    )
    .await;

    let manifest = common::read_file(project.path(), "app/src/main/AndroidManifest.xml");
    assert!(
        manifest.contains(r#"android:name="com.example.droid.work.SyncService""#),
        "{manifest}"
    );
    let graph = common::read_file(project.path(), &format!("{RES}/navigation/nav_main.xml"));
    assert!(
        graph.contains(r#"android:name="com.example.droid.home.HomeFragment""#),
        "{graph}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_class_renamed_with_its_file_is_renamed_in_xml() {
    let project = setup().await;

    run(
        project.path(),
        &[(app("widgets/BadgeView.kt"), app("widgets/CounterView.kt"))],
    )
    .await;

    let layout = common::read_file(project.path(), &format!("{RES}/layout/activity_main.xml"));
    assert!(
        layout.contains("<com.example.droid.widgets.CounterView"),
        "{layout}"
    );
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn an_android_class_rename_updates_the_layout_tag() {
    let project = setup().await;
    let view = app("widgets/BadgeView.kt");

    let report = rename(request(project.path(), &view, "BadgeView", "CounterView")).await;

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
    project.assert_compiles(COMPILE);
}

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
