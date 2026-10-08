use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::Project;

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn the_plan_lists_the_files_that_travel_with_the_package() {
    let project = Project::from_fixture("go/project");
    let plan = assert_plan_matches_move(
        &project,
        &[("pkg/utils/format.go", "pkg/helpers/format.go")],
    );
    let moved: Vec<&str> = plan["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["from"].as_str().unwrap())
        .collect();
    assert!(
        moved.contains(&"pkg/utils/validate.go"),
        "gopls moves the whole package, so validate.go moves too: {plan}"
    );
    assert!(
        plan["notes"].to_string().contains("entire packages"),
        "{plan}"
    );
}
