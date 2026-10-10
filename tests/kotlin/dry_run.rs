use crate::common;
use crate::common::dry_run::{assert_plan_was_carried_out, plan_on_fresh_copy, tree_without};
use crate::common::project::Project;
use refac::drivers::kotlin::moves::move_files;

// `refac move --dry-run` for Kotlin runs the real move on a throw-away copy of
// the project and reports the difference; the real move on the original then
// has to do exactly that. Gradle and the server write `.gradle`, `.kotlin` and
// `build` folders into the project, which are not part of the comparison.
// The plan is made by the real binary on a copy (a server of its own, 45 seconds); the
// real move that it is compared with runs on the shared server of the fixture
// (common::pool), which takes seconds.
// Run with: REFAC_KOTLIN_SERVER=<install dir> cargo test --test kotlin dry_run:: -- --ignored --test-threads=1

const K: &str = "src/main/kotlin/com/example";
const SCRATCH: &[&str] = &[".gradle", ".kotlin", ".idea", "build", "app/build"];

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn the_plan_of_a_package_move_names_every_file_whose_import_changes() {
    let leased = common::pool::lease("kotlin/jvm_project").await;
    let project = Project::over(leased.path());
    let from = format!("{K}/util/Helper.kt");
    let to = format!("{K}/common/Helper.kt");

    let plan = plan_on_fresh_copy("kotlin/jvm_project", &[(&from, &to)], SCRATCH);
    let before = tree_without(&project, SCRATCH);
    move_files(&[(from, to)], Some(leased.path()))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"));
    assert_plan_was_carried_out(&project, &plan, &before, SCRATCH);

    // The moved file itself (its package line) and its importers.
    assert!(plan["edited_files"].as_u64().unwrap() >= 4, "{plan}");
    // One Kotlin file on its own: the plan says what that costs and that the
    // real run pays it again.
    let notes = plan["notes"].to_string();
    assert!(notes.contains("This Kotlin move took"), "{notes}");
    assert!(notes.contains("This was a dry run"), "{notes}");
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and the Android SDK (ANDROID_HOME)"]
async fn the_plan_of_an_android_move_includes_the_manifest_and_layout_edits() {
    common::kotlin::require_android_sdk();
    let leased = common::pool::lease("kotlin/android_project").await;
    let project = Project::over(leased.path());
    let base = "app/src/main/java/com/example/droid";
    let from = format!("{base}/MainActivity.kt");
    let to = format!("{base}/ui/MainActivity.kt");

    let plan = plan_on_fresh_copy("kotlin/android_project", &[(&from, &to)], SCRATCH);
    let before = tree_without(&project, SCRATCH);
    move_files(&[(from, to)], Some(leased.path()))
        .await
        .unwrap_or_else(|error| panic!("the move failed: {error:#}"));
    assert_plan_was_carried_out(&project, &plan, &before, SCRATCH);

    let files = plan["files"].to_string();
    assert!(files.contains("AndroidManifest.xml"), "{plan}");
    assert!(files.contains("activity_main.xml"), "{plan}");
}
