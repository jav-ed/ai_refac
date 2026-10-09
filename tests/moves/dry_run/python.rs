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

#[test]
#[ignore = "needs python3 with the rope package (pip install rope)"]
fn a_dry_run_copies_python_files_only_so_data_files_do_not_count_against_the_limit() {
    let project = Project::from_fixture("python/project");
    // Two MiB of data next to the code, and a limit of one MiB for the copy.
    project.write_bytes("data/model.bin", &vec![7u8; 2 * 1024 * 1024]);
    let output = project.dry_run_json_with_env(
        &[("myapp/utils/formatters.py", "myapp/core/formatters.py")],
        &[("REFAC_DRY_RUN_COPY_MAX_MB", "1")],
    );
    assert!(
        output.status.success(),
        "{}",
        crate::common::stderr_text(&output)
    );
    assert!(project.exists("data/model.bin"));
}
