//! The package as a whole: its files move together, its declaration changes, and unrelated packages stay as they are.

use super::run_move;
use crate::common;

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_moves_validate_go_to_helpers() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // gopls renames the whole package — validate.go moves alongside format.go
    assert!(
        project.join("pkg/helpers/validate.go").exists(),
        "validate.go must move to pkg/helpers/ (gopls renames the whole package)"
    );
    assert!(
        !project.join("pkg/utils/validate.go").exists(),
        "validate.go must no longer be in pkg/utils/ after the package rename"
    );
    let validate = common::read_file(project, "pkg/helpers/validate.go");
    assert!(
        validate.contains("package helpers"),
        "validate.go package declaration must be helpers:\n{validate}"
    );
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_updates_package_declaration_in_moved_file() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let moved = common::read_file(project, "pkg/helpers/format.go");
    assert!(
        !moved.contains("package utils"),
        "old package declaration should be gone:\n{moved}"
    );
    assert!(
        moved.contains("package helpers"),
        "new package declaration missing:\n{moved}"
    );
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_does_not_touch_blank_import_of_unrelated_package() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // The blank import of pkg/setup is unrelated — it must not be touched.
    let main = common::read_file(project, "cmd/main.go");
    assert!(
        main.contains("_ \"github.com/example/myproject/pkg/setup\""),
        "blank import of unrelated package should be preserved:\n{main}"
    );
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_leaves_control_file_unchanged() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "config/config.go");
    common::assert_move_succeeded(&run_move(project));

    let after = common::read_file(project, "config/config.go");
    assert_eq!(
        after, before,
        "config/config.go has no pkg/utils dependency — must be byte-identical after move"
    );
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_leaves_setup_package_unchanged() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "pkg/setup/init.go");
    common::assert_move_succeeded(&run_move(project));

    let after = common::read_file(project, "pkg/setup/init.go");
    assert_eq!(
        after, before,
        "pkg/setup/init.go is the blank-import target — must be byte-identical after move"
    );
}
