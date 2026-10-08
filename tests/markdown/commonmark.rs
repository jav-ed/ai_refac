use crate::common;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

// Behaviours that follow from reading Markdown as CommonMark and from where the
// project path points. Moves under test: docs/guide.md -> docs/manual/guide.md.

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn move_guide(cwd: &Path, project_path: &str) -> Output {
    Command::new(common::cli_binary())
        .current_dir(cwd)
        .args([
            "move",
            "--project-path",
            project_path,
            "--source-path",
            "docs/guide.md",
            "--target-path",
            "docs/manual/guide.md",
        ])
        .output()
        .expect("failed to execute CLI binary")
}

#[test]
fn a_relative_project_path_updates_links_like_an_absolute_one() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    write(project, "README.md", "See [guide](docs/guide.md).\n");
    write(project, "docs/guide.md", "# Guide\n");

    // `.` is resolved against the working directory, which is the project.
    common::assert_move_succeeded(&move_guide(project, "."));

    assert!(project.join("docs/manual/guide.md").exists());
    let readme = common::read_file(project, "README.md");
    assert!(
        readme.contains("(docs/manual/guide.md)"),
        "the link must follow the move:\n{readme}"
    );
}

#[test]
fn only_real_links_are_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    let original = "\
[link](docs/guide.md)

    indented code: [x](docs/guide.md)

- item

      deeper code [y](docs/guide.md)

<!-- [c](docs/guide.md) -->

<a href=\"docs/guide.md\">html</a>

[ref]: docs/guide.md
";
    write(project, "README.md", original);
    write(project, "docs/guide.md", "# Guide\n");

    common::assert_move_succeeded(&move_guide(project, project.to_str().unwrap()));

    let expected = original
        .replace("[link](docs/guide.md)", "[link](docs/manual/guide.md)")
        .replace(
            "<a href=\"docs/guide.md\">",
            "<a href=\"docs/manual/guide.md\">",
        )
        .replace("[ref]: docs/guide.md", "[ref]: docs/manual/guide.md");
    assert_eq!(
        common::read_file(project, "README.md"),
        expected,
        "code blocks and comments keep their text; links, HTML references, and definitions follow"
    );
}

#[test]
fn a_definition_on_two_lines_is_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    write(
        project,
        "README.md",
        "See [guide].\n\n[guide]:\n   docs/guide.md\n",
    );
    write(project, "docs/guide.md", "# Guide\n");

    common::assert_move_succeeded(&move_guide(project, project.to_str().unwrap()));

    let readme = common::read_file(project, "README.md");
    assert!(
        readme.contains("[guide]:\n   docs/manual/guide.md\n"),
        "{readme}"
    );
}
