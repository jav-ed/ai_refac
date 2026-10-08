use crate::common::DART_LOCK;
use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::Project;

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn the_plan_names_the_files_whose_imports_change() {
    let _lock = DART_LOCK.lock().unwrap();
    let project = Project::from_fixture("dart/project");
    let plan = assert_plan_matches_move(
        &project,
        &[("lib/src/formatter.dart", "lib/src/core/formatter.dart")],
    );
    assert!(plan["edits"].as_u64().unwrap() >= 1, "{plan}");
}
