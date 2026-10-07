#[allow(dead_code)]
mod common;

// Real Kotlin language server against tests/fixtures/kotlin/android_project
// (namespace com.example.droid, sources in src/main/java):
//   MainActivity.kt        in the namespace package, uses R, named by manifest and layout
//   widgets/BadgeView.kt   custom view used as a layout tag and from Launcher.java
//   sync/SyncService.kt    named by its full name in the manifest
//   ui/HomeFragment.kt     named by the navigation graph
// The server never edits XML and drops the implicit R when a file leaves the
// namespace package; these tests prove refac repairs both, judged by a compile.
// Run with: REFAC_KOTLIN_SERVER=<install dir> ANDROID_HOME=<sdk> \
//   cargo test --test kotlin_android -- --ignored

use refac::drivers::kotlin::moves::{MoveReport, move_files};
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

fn setup() -> tempfile::TempDir {
    common::kotlin::require_server();
    common::kotlin::require_android_sdk();
    common::setup_fixture("kotlin/android_project")
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn an_activity_moves_and_manifest_layout_and_r_follow() {
    let project = setup();

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
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_custom_view_directory_is_renamed_and_its_layout_tag_follows() {
    let project = setup();

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
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_service_and_a_fragment_are_renamed_in_manifest_and_navigation_graph() {
    let project = setup();

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
    common::kotlin::assert_compiles(project.path(), COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn a_class_renamed_with_its_file_is_renamed_in_xml() {
    let project = setup();

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
    common::kotlin::assert_compiles(project.path(), COMPILE);
}
