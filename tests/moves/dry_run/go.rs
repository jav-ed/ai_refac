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

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn a_batch_of_packages_is_planned_as_the_real_move_does_it() {
    let project = Project::from_fixture("go/project");
    // `models` imports `utils`, and both packages move in one command.
    assert_plan_matches_move(
        &project,
        &[
            ("pkg/utils/format.go", "pkg/helpers/format.go"),
            ("pkg/models/user.go", "pkg/entities/user.go"),
        ],
    );
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn a_batch_of_four_packages_that_import_each_other_is_planned_as_the_real_move_does_it() {
    let project = Project::from_fixture("go/project");
    // services imports models and utils, api imports services: the packages
    // that move depend on each other and on packages that move as well.
    assert_plan_matches_move(
        &project,
        &[
            ("pkg/utils/format.go", "pkg/helpers/format.go"),
            ("pkg/models/user.go", "pkg/entities/user.go"),
            ("pkg/services/user_service.go", "pkg/svc/user_service.go"),
            ("pkg/api/router.go", "pkg/http/router.go"),
        ],
    );
}
