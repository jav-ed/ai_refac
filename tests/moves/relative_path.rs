//! `--project-path` written relative to the working directory, as scripts and
//! agents write it. Every driver joins the relative file paths onto the
//! project folder, so a folder that is itself relative must be made absolute
//! once: Go used to read `<project>/<project>/pkg/...` and fail.

use crate::common::assert_move_succeeded;
use crate::common::project::Project;

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_moves_a_package_from_a_relative_project_path() {
    let project = Project::from_fixture("go/project");
    let output = project.run_from_parent(&[("pkg/utils/format.go", "pkg/helpers/format.go")], &[]);
    assert_move_succeeded(&output);
    assert!(project.exists("pkg/helpers/format.go"));
    assert!(!project.exists("pkg/utils/format.go"));
    assert!(project.read("cmd/main.go").contains("pkg/helpers"));
}

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_plans_a_move_from_a_relative_project_path() {
    let project = Project::from_fixture("go/project");
    let before = project.tree();
    let output = project.run_from_parent(
        &[("pkg/utils/format.go", "pkg/helpers/format.go")],
        &["--dry-run"],
    );
    assert_move_succeeded(&output);
    crate::common::project::assert_same_tree(&before, &project.tree());
}

#[test]
fn typescript_moves_a_file_from_a_relative_project_path() {
    let project = Project::from_fixture("typescript/project");
    let output =
        project.run_from_parent(&[("src/utils/math_helpers.ts", "src/utils/maths.ts")], &[]);
    assert_move_succeeded(&output);
    assert!(project.exists("src/utils/maths.ts"));
    assert!(!project.exists("src/utils/math_helpers.ts"));
}

#[test]
fn markdown_updates_links_from_a_relative_project_path() {
    let project = Project::empty();
    project.write("docs/a.md", "# A\n\n[b](b.md)\n");
    project.write("docs/b.md", "# B\n");
    let output = project.run_from_parent(&[("docs/b.md", "docs/guides/b.md")], &[]);
    assert_move_succeeded(&output);
    assert!(project.read("docs/a.md").contains("guides/b.md"));
}
