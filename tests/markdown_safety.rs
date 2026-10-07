//! What the Markdown backend refuses, leaves alone, and undoes.

mod common;

use common::project::{Project, assert_same_tree};

fn two_files() -> Project {
    let project = Project::empty();
    project.write("README.md", "See [guide](docs/guide.md).\n");
    project.write("docs/guide.md", "# Guide\n[readme](../README.md)\n");
    project
}

#[test]
fn folders_git_and_node_modules_never_see_a_change() {
    let project = two_files();
    project.write(
        "node_modules/pkg/README.md",
        "[guide](../../docs/guide.md)\n",
    );
    project.write("build/report.md", "[guide](../docs/guide.md)\n");
    project.write(".gitignore", "build/\n");
    project.write(".git/info/notes.md", "[guide](../../docs/guide.md)\n");
    let before = project.tree();

    project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);
    let after = project.tree();

    assert_eq!(
        after
            .get("README.md")
            .map(|b| String::from_utf8_lossy(b).into_owned()),
        Some("See [guide](docs/manual/guide.md).\n".to_string())
    );
    for untouched in [
        "node_modules/pkg/README.md",
        "build/report.md",
        ".git/info/notes.md",
        ".gitignore",
    ] {
        assert_eq!(after.get(untouched), before.get(untouched), "{untouched}");
    }
}

#[test]
fn a_file_that_is_not_utf8_is_reported_and_left_alone() {
    let project = two_files();
    project.write_bytes(
        "legacy.md",
        b"[guide](docs/guide.md) \xff\xfe latin-1 \xe4\n",
    );

    let output = project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);

    assert!(output.contains("not valid UTF-8"), "{output}");
    assert!(output.contains("legacy.md"), "{output}");
    assert_eq!(
        std::fs::read(project.path().join("legacy.md")).unwrap(),
        b"[guide](docs/guide.md) \xff\xfe latin-1 \xe4\n"
    );
    assert_eq!(
        project.read("README.md"),
        "See [guide](docs/manual/guide.md).\n"
    );
}

#[test]
fn line_endings_and_a_byte_order_mark_survive() {
    let project = two_files();
    project.write(
        "crlf.md",
        "\u{feff}Line one\r\n[guide](docs/guide.md)\r\n\r\nEnd\r\n",
    );

    project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);

    assert_eq!(
        project.read("crlf.md"),
        "\u{feff}Line one\r\n[guide](docs/manual/guide.md)\r\n\r\nEnd\r\n"
    );
}

#[test]
fn a_link_to_the_file_itself_and_a_spelling_with_dots_are_handled() {
    let project = two_files();
    project.write(
        "docs/self.md",
        "[me](self.md) [me too](./self.md#top) [odd](../docs/guide.md)\n",
    );

    project.move_ok(&[
        ("docs/self.md", "docs/deep/self.md"),
        ("docs/guide.md", "docs/deep/guide.md"),
    ]);

    // Both files moved into the same folder: the link to itself says the same,
    // and the odd spelling now says what it means in one step.
    assert_eq!(
        project.read("docs/deep/self.md"),
        "[me](self.md) [me too](./self.md#top) [odd](guide.md)\n"
    );
}

#[test]
fn the_extensions_markdown_and_mdx_and_capitals_are_documents_too() {
    let project = Project::empty();
    project.write("A.MD", "[b](notes/b.markdown) [c](notes/c.mdx)\n");
    project.write("notes/b.markdown", "[a](../A.MD)\n");
    project.write("notes/c.mdx", "[a](../A.MD)\n");

    project.move_ok(&[("A.MD", "top/A.MD")]);

    assert_eq!(
        project.read("top/A.MD"),
        "[b](../notes/b.markdown) [c](../notes/c.mdx)\n"
    );
    assert_eq!(project.read("notes/b.markdown"), "[a](../top/A.MD)\n");
    assert_eq!(project.read("notes/c.mdx"), "[a](../top/A.MD)\n");
}

#[test]
fn an_existing_target_is_refused_and_nothing_changes() {
    let project = two_files();
    project.write("docs/other.md", "# Other\n");
    let before = project.tree();

    let error = project.move_err(&[("docs/guide.md", "docs/other.md")]);

    assert!(error.contains("already exists"), "{error}");
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_markdown_file_must_stay_a_markdown_file() {
    let project = two_files();
    let before = project.tree();

    let error = project.move_err(&[("docs/guide.md", "docs/guide.txt")]);

    assert!(error.contains("must stay a Markdown file"), "{error}");
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_folder_cannot_move_into_itself() {
    let project = two_files();
    let before = project.tree();

    let error = project.move_err(&[("docs", "docs/old")]);

    assert!(error.contains("into itself"), "{error}");
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_folder_with_code_is_not_a_document_folder() {
    let project = two_files();
    project.write("docs/tool.sh", "echo hi\n");
    let before = project.tree();

    let error = project.move_err(&[("docs", "documentation")]);

    assert!(
        error.contains("holds none of those, or holds code of another language"),
        "{error}"
    );
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_missing_source_is_refused() {
    let project = two_files();

    let error = project.move_err(&[("docs/nope.md", "docs/yes.md")]);

    assert!(error.contains("does not exist"), "{error}");
}

#[test]
fn a_move_that_fails_halfway_puts_everything_back() {
    let project = two_files();
    project.write("a.md", "# A\n");
    project.write("b.md", "# B\n");
    // `blocked` is a file, so nothing can be created below it.
    project.write("blocked", "I am a file\n");
    let before = project.tree();

    let error = project.move_err(&[("a.md", "moved/a.md"), ("b.md", "blocked/b.md")]);

    assert!(error.contains("rolled back"), "{error}");
    assert_same_tree(&before, &project.tree());
}

#[test]
fn nothing_to_update_is_not_an_error() {
    let project = Project::empty();
    project.write("lonely.md", "# Alone\n");

    let output = project.move_ok(&[("lonely.md", "docs/lonely.md")]);

    assert!(output.contains("updated 0 links in 0 files"), "{output}");
    assert_eq!(project.read("docs/lonely.md"), "# Alone\n");
}
