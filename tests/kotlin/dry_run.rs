use crate::common;
use crate::common::dry_run::assert_plan_matches_move_ignoring;
use crate::common::project::Project;

// `refac move --dry-run` for Kotlin runs the real move on a throw-away copy of
// the project and reports the difference; the real move on the original then
// has to do exactly that. Gradle and the server write `.gradle`, `.kotlin` and
// `build` folders into the project, which are not part of the comparison.
// Run with: REFAC_KOTLIN_SERVER=<install dir> cargo test --test kotlin dry_run:: -- --ignored

const K: &str = "src/main/kotlin/com/example";
const SCRATCH: &[&str] = &[".gradle", ".kotlin", ".idea", "build", "app/build"];

#[test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
fn the_plan_of_a_package_move_names_every_file_whose_import_changes() {
    common::kotlin::require_server();
    let project = Project::from_fixture("kotlin/jvm_project");
    let from = format!("{K}/util/Helper.kt");
    let to = format!("{K}/common/Helper.kt");

    let plan = assert_plan_matches_move_ignoring(&project, &[(&from, &to)], SCRATCH);

    // The moved file itself (its package line) and its importers.
    assert!(plan["edited_files"].as_u64().unwrap() >= 4, "{plan}");
}

#[test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
fn the_plan_of_an_android_move_includes_the_manifest_and_layout_edits() {
    common::kotlin::require_server();
    common::kotlin::require_android_sdk();
    let project = Project::from_fixture("kotlin/android_project");
    let base = "app/src/main/java/com/example/droid";
    let from = format!("{base}/MainActivity.kt");
    let to = format!("{base}/ui/MainActivity.kt");

    let plan = assert_plan_matches_move_ignoring(&project, &[(&from, &to)], SCRATCH);

    let files = plan["files"].to_string();
    assert!(files.contains("AndroidManifest.xml"), "{plan}");
    assert!(files.contains("activity_main.xml"), "{plan}");
}

#[test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
fn a_dry_run_leaves_no_server_or_gradle_daemon_behind() {
    common::kotlin::require_server();
    let project = Project::from_fixture("kotlin/jvm_project");
    let from = format!("{K}/util/Helper.kt");
    let to = format!("{K}/common/Helper.kt");
    let output = project.dry_run_json(&[(&from, &to)]);
    assert!(output.status.success());
    // The original was never opened by Gradle: nothing of the tools' own state.
    assert!(!project.exists(".gradle"));
    assert!(!project.exists("build"));
    assert!(project.exists(&from));
}
