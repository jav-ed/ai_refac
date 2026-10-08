use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::Project;

#[test]
#[ignore = "needs python3 with the rope package (pip install rope)"]
fn the_plan_of_a_rope_move_is_the_difference_to_a_copy() {
    let project = Project::from_fixture("python/project");
    let plan = assert_plan_matches_move(
        &project,
        &[("myapp/utils/formatters.py", "myapp/core/formatters.py")],
    );
    assert!(plan["edited_files"].as_u64().unwrap() >= 5, "{plan}");
}

#[test]
#[ignore = "needs python3 with the rope package (pip install rope)"]
fn rope_leaves_no_state_folder_in_the_project() {
    let project = Project::from_fixture("python/project");
    let output = project.dry_run_json(&[("myapp/utils/formatters.py", "myapp/core/formatters.py")]);
    assert!(output.status.success());
    assert!(!project.exists(".ropeproject"));
}
