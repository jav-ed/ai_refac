use crate::common::dry_run::assert_plan_matches_move_ignoring;
use crate::common::project::Project;

#[test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
fn the_plan_of_a_module_file_rename_names_the_declaration_and_the_users() {
    let project = Project::from_fixture("rust/rename_crate");
    // rust-analyzer's own `cargo check` fills target/, which is the tool's
    // scratch and not part of the move.
    let plan = assert_plan_matches_move_ignoring(
        &project,
        &[("src/shapes.rs", "src/figures.rs")],
        &["target"],
    );
    assert!(plan["edited_files"].as_u64().unwrap() >= 1, "{plan}");
}

#[test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
fn a_lock_file_cargo_writes_while_the_project_loads_is_removed_again() {
    let project = Project::from_fixture("rust/rename_crate");
    assert!(
        !project.exists("Cargo.lock"),
        "the fixture has no lock file"
    );
    let output = project.dry_run_json(&[("src/shapes.rs", "src/figures.rs")]);
    assert!(output.status.success());
    assert!(!project.exists("Cargo.lock"));
    assert!(project.exists("src/shapes.rs"));
    assert!(!project.exists("src/figures.rs"));
}
