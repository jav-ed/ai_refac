mod common;

#[test]
fn cross_directory_rust_file_move_requires_semantic_module_command() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();
    let source = project.join("src/types.rs");
    let target = project.join("src/shared/types.rs");
    let before = common::read_file(project, "src/types.rs");

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        source.to_str().unwrap(),
        "--target-path",
        target.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let error = common::stderr_text(&output);
    assert!(error.contains("move-module"), "{error}");
    assert!(
        error.contains("without introducing #[path] shims"),
        "{error}"
    );
    assert_eq!(common::read_file(project, "src/types.rs"), before);
    assert!(!target.exists());
}

#[test]
fn path_attribute_module_is_rejected_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    std::fs::create_dir_all(project.join("src/legacy")).unwrap();
    std::fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"legacy\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/lib.rs"),
        "#[path = \"legacy/value.rs\"]\npub mod value;\n",
    )
    .unwrap();
    std::fs::write(project.join("src/legacy/value.rs"), "pub fn value() {}\n").unwrap();

    let output = common::run_cli(&[
        "move-module",
        "--project-path",
        project.to_str().unwrap(),
        "crate::value",
        "crate::domain::value",
    ]);

    assert!(!output.status.success());
    let error = common::stderr_text(&output);
    assert!(error.contains("uses #[path]"), "{error}");
    assert!(project.join("src/legacy/value.rs").exists());
    assert!(!project.join("src/domain").exists());
}

#[test]
fn failed_post_move_check_rolls_back_every_source_change() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    std::fs::create_dir_all(project.join("src/parent")).unwrap();
    std::fs::create_dir_all(project.join("src/domain")).unwrap();
    std::fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"rollback\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/lib.rs"),
        "pub mod domain;\npub mod parent;\n",
    )
    .unwrap();
    std::fs::write(project.join("src/domain/mod.rs"), "").unwrap();
    std::fs::write(
        project.join("src/parent/mod.rs"),
        "pub mod moving;\nmod private;\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/parent/moving.rs"),
        "use super::private;\npub fn run() { private::hidden(); }\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/parent/private.rs"),
        "pub fn hidden() {}\n",
    )
    .unwrap();
    let parent_before = common::read_file(project, "src/parent/mod.rs");
    let moving_before = common::read_file(project, "src/parent/moving.rs");

    let output = common::run_cli(&[
        "move-module",
        "--project-path",
        project.to_str().unwrap(),
        "crate::parent::moving",
        "crate::domain::moving",
    ]);

    assert!(!output.status.success());
    let error = common::stderr_text(&output);
    assert!(error.contains("every changed path was restored"), "{error}");
    assert_eq!(
        common::read_file(project, "src/parent/mod.rs"),
        parent_before
    );
    assert_eq!(
        common::read_file(project, "src/parent/moving.rs"),
        moving_before
    );
    assert!(!project.join("src/domain/moving.rs").exists());
    assert_eq!(common::read_file(project, "src/domain/mod.rs"), "");
}
