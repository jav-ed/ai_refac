//! Markdown links to code files. When a language backend moves a file, the
//! README that points at it must follow, whichever language the file is in.
//! The projects are the move fixtures of each language plus a README and a
//! guide written here.

use crate::common;

use common::project::Project;

const NOTE: &str = "Markdown links to the moved files";

fn with_docs(fixture: &str, readme: &str, guide: &str) -> Project {
    let project = Project::from_fixture(fixture);
    project.write("README.md", readme);
    project.write("docs/guide.md", guide);
    project
}

#[test]
fn a_typescript_file_move_updates_the_links_to_it() {
    let project = with_docs(
        "typescript/project",
        "Dates live in [date_helpers](src/utils/date_helpers.ts#L1) and `src/utils/date_helpers.ts`.\n",
        "See [the helpers](../src/utils/date_helpers.ts) and [the index](../src/utils/index.ts).\n",
    );

    let output = project.move_ok(&[("src/utils/date_helpers.ts", "src/lib/date_helpers.ts")]);

    assert!(output.contains(NOTE), "{output}");
    assert!(
        output.contains("Checked 2 Markdown files; updated 2 links in 2 files."),
        "{output}"
    );
    assert_eq!(
        project.read("README.md"),
        "Dates live in [date_helpers](src/lib/date_helpers.ts#L1) and `src/utils/date_helpers.ts`.\n"
    );
    assert_eq!(
        project.read("docs/guide.md"),
        "See [the helpers](../src/lib/date_helpers.ts) and [the index](../src/utils/index.ts).\n"
    );
}

#[test]
fn a_typescript_folder_move_updates_links_to_the_folder_and_to_files_in_it() {
    let project = with_docs(
        "typescript/project",
        "The [utilities](src/utils/) and the [math](src/utils/math_helpers.ts).\n",
        "Only [the config](../src/config.ts) stays.\n",
    );

    project.move_ok(&[("src/utils", "src/helpers")]);

    assert_eq!(
        project.read("README.md"),
        "The [utilities](src/helpers/) and the [math](src/helpers/math_helpers.ts).\n"
    );
    assert_eq!(
        project.read("docs/guide.md"),
        "Only [the config](../src/config.ts) stays.\n"
    );
}

#[test]
fn a_python_file_move_updates_the_links_to_it() {
    let project = with_docs(
        "python/project",
        "[formatters](myapp/utils/formatters.py)\n",
        "[formatters](../myapp/utils/formatters.py)\n",
    );

    let output = project.move_ok(&[("myapp/utils/formatters.py", "myapp/core/formatters.py")]);

    assert!(output.contains(NOTE), "{output}");
    assert_eq!(
        project.read("README.md"),
        "[formatters](myapp/core/formatters.py)\n"
    );
    assert_eq!(
        project.read("docs/guide.md"),
        "[formatters](../myapp/core/formatters.py)\n"
    );
}

#[test]
fn a_go_file_move_updates_the_links_to_it() {
    let project = with_docs(
        "go/project",
        "[format](pkg/utils/format.go)\n",
        "[format](../pkg/utils/format.go)\n",
    );

    project.move_ok(&[("pkg/utils/format.go", "pkg/helpers/format.go")]);

    assert_eq!(
        project.read("README.md"),
        "[format](pkg/helpers/format.go)\n"
    );
    assert_eq!(
        project.read("docs/guide.md"),
        "[format](../pkg/helpers/format.go)\n"
    );
}

#[test]
fn a_dart_file_move_updates_the_links_to_it() {
    let project = with_docs(
        "dart/project",
        "[formatter](lib/src/formatter.dart)\n",
        "[formatter](../lib/src/formatter.dart)\n",
    );

    project.move_ok(&[("lib/src/formatter.dart", "lib/src/core/formatter.dart")]);

    assert_eq!(
        project.read("README.md"),
        "[formatter](lib/src/core/formatter.dart)\n"
    );
    assert_eq!(
        project.read("docs/guide.md"),
        "[formatter](../lib/src/core/formatter.dart)\n"
    );
}

#[test]
fn a_request_with_code_markdown_and_an_image_updates_every_link_once() {
    let project = with_docs(
        "typescript/project",
        "![logo](docs/logo.png) [guide](docs/guide.md) [dates](src/utils/date_helpers.ts)\n",
        "![logo](logo.png) [dates](../src/utils/date_helpers.ts) [readme](../README.md)\n",
    );
    project.write_bytes("docs/logo.png", b"\x89PNG-logo");

    let output = project.move_ok(&[
        ("src/utils/date_helpers.ts", "src/lib/date_helpers.ts"),
        ("docs/guide.md", "manual/guide.md"),
        ("docs/logo.png", "manual/logo.png"),
    ]);

    assert!(output.contains(NOTE), "{output}");
    assert_eq!(
        project.read("README.md"),
        "![logo](manual/logo.png) [guide](manual/guide.md) [dates](src/lib/date_helpers.ts)\n"
    );
    assert_eq!(
        project.read("manual/guide.md"),
        "![logo](logo.png) [dates](../src/lib/date_helpers.ts) [readme](../README.md)\n"
    );
}

#[test]
fn a_failed_code_move_leaves_the_markdown_links_alone() {
    let project = with_docs(
        "rust/project",
        "[types](src/types.rs)\n",
        "[types](../src/types.rs)\n",
    );

    // Rust refuses a move to another folder; the link must not be rewritten.
    let error = project.move_err(&[("src/types.rs", "src/shared/types.rs")]);

    assert!(error.contains("move-module"), "{error}");
    assert_eq!(project.read("README.md"), "[types](src/types.rs)\n");
}
