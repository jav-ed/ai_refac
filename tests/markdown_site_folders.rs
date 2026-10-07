//! Moves of folders, images, and batches in the documentation site
//! (`tests/fixtures/markdown/site`). Links into a folder follow it, links
//! inside it stay, and every Markdown file keeps linking to the same files.

mod common;

use common::links::edges;
use common::project::{assert_same_tree, text};
use common::site::{assert_links_follow, edited, site};

#[test]
fn moving_a_folder_updates_links_into_it_and_leaves_links_inside_it_alone() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[("docs", "documentation")]);
    let after = project.tree();

    // Inside the folder every file keeps its text: nothing it links to left it,
    // and the readme it links to is still one folder up.
    for (path, bytes) in &before {
        if let Some(rest) = path.strip_prefix("docs/") {
            assert_eq!(
                after.get(&format!("documentation/{rest}")),
                Some(bytes),
                "{path}"
            );
        }
    }
    assert_eq!(
        text(&after, "README.md"),
        edited(
            &before,
            "README.md",
            &[
                (
                    "(docs/guide.md#install)",
                    "(documentation/guide.md#install)"
                ),
                ("(docs/api/overview.md)", "(documentation/api/overview.md)"),
                (
                    "![Logo](docs/img/logo.png)",
                    "![Logo](documentation/img/logo.png)"
                ),
                (
                    "<img src=\"docs/img/logo.png\"",
                    "<img src=\"documentation/img/logo.png\""
                ),
                ("[all docs](docs/)", "[all docs](documentation/)"),
                ("(<docs/My Notes.md>)", "(<documentation/My Notes.md>)"),
                (
                    "(docs/My%20Notes.md?plain=1)",
                    "(documentation/My%20Notes.md?plain=1)"
                ),
                ("(docs/missing.md)", "(documentation/missing.md)"),
                (
                    "[diagram]: docs/img/diagram.svg",
                    "[diagram]: documentation/img/diagram.svg"
                ),
            ]
        )
    );
    assert_eq!(
        text(&after, "CHANGELOG.md"),
        edited(
            &before,
            "CHANGELOG.md",
            &[
                ("(docs/guide.md)", "(documentation/guide.md)"),
                (
                    "(docs/api/endpoints.md#list)",
                    "(documentation/api/endpoints.md#list)"
                ),
            ]
        )
    );
    assert_eq!(
        text(&after, ".github/CONTRIBUTING.md"),
        edited(
            &before,
            ".github/CONTRIBUTING.md",
            &[("(../docs/guide.md)", "(../documentation/guide.md)")]
        )
    );
    assert_links_follow(&before, &after, &|path| match path.strip_prefix("docs") {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("documentation{rest}"),
        _ => path.to_string(),
    });

    project.move_ok(&[("documentation", "docs")]);
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_folder_that_moves_deeper_fixes_the_links_that_leave_it() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[("docs", "content/docs")]);
    let after = project.tree();

    assert_eq!(
        text(&after, "content/docs/index.md"),
        edited(
            &before,
            "docs/index.md",
            &[("(../README.md)", "(../../README.md)")]
        )
    );
    assert_eq!(
        text(&after, "content/docs/api/overview.md"),
        edited(
            &before,
            "docs/api/overview.md",
            &[("(../../README.md)", "(../../../README.md)")]
        )
    );
    assert_eq!(
        text(&after, "content/docs/guide.md"),
        edited(
            &before,
            "docs/guide.md",
            &[("(../README.md)", "(../../README.md)")]
        )
    );
    assert_links_follow(&before, &after, &|path| {
        if path.starts_with("docs") {
            format!("content/{path}")
        } else {
            path.to_string()
        }
    });

    project.move_ok(&[("content/docs", "docs")]);
    let restored = project.tree();
    // `content/` is an empty folder now; the files are what counts.
    assert_same_tree(&before, &restored);
}

#[test]
fn moving_an_image_updates_markdown_and_html_references_and_keeps_its_bytes() {
    let project = site();
    let before = project.tree();

    let output = project.move_ok(&[("docs/img/logo.png", "assets/logo.png")]);
    let after = project.tree();

    assert!(output.contains("Markdown results"), "{output}");
    assert_eq!(
        after.get("assets/logo.png"),
        before.get("docs/img/logo.png")
    );
    assert_eq!(
        text(&after, "README.md"),
        edited(
            &before,
            "README.md",
            &[
                ("![Logo](docs/img/logo.png)", "![Logo](assets/logo.png)"),
                (
                    "<img src=\"docs/img/logo.png\"",
                    "<img src=\"assets/logo.png\""
                ),
            ]
        )
    );
    assert_eq!(
        text(&after, "docs/index.md"),
        edited(
            &before,
            "docs/index.md",
            &[("![Logo](img/logo.png)", "![Logo](../assets/logo.png)")]
        )
    );
    assert_eq!(
        text(&after, "docs/page.mdx"),
        edited(
            &before,
            "docs/page.mdx",
            &[(
                "<img src=\"./img/logo.png\" />",
                "<img src=\"../assets/logo.png\" />"
            )]
        ),
        "the import line is code, not a link, and stays"
    );
    assert_links_follow(&before, &after, &|path| {
        if path == "docs/img/logo.png" {
            "assets/logo.png".to_string()
        } else {
            path.to_string()
        }
    });

    // Moving back cannot know that `docs/page.mdx` once began its path with
    // `./`: a path that climbs has none, so it comes back without. Everything
    // else is as it was.
    project.move_ok(&[("assets/logo.png", "docs/img/logo.png")]);
    let mut expected = before.clone();
    expected.insert(
        "docs/page.mdx".to_string(),
        edited(
            &before,
            "docs/page.mdx",
            &[(
                "<img src=\"./img/logo.png\" />",
                "<img src=\"img/logo.png\" />",
            )],
        )
        .into_bytes(),
    );
    assert_same_tree(&expected, &project.tree());
}

#[test]
fn a_batch_of_files_in_different_places_keeps_every_link_between_them_right() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[
        ("docs/guide.md", "tutorials/guide.md"),
        ("docs/index.md", "docs/home.md"),
        ("docs/img", "docs/images"),
        ("README.md", "docs/README.md"),
    ]);
    let after = project.tree();

    assert_links_follow(&before, &after, &|path| match path {
        "docs/guide.md" => "tutorials/guide.md".to_string(),
        "docs/index.md" => "docs/home.md".to_string(),
        "README.md" => "docs/README.md".to_string(),
        _ => match path.strip_prefix("docs/img") {
            Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("docs/images{rest}"),
            _ => path.to_string(),
        },
    });

    project.move_ok(&[
        ("tutorials/guide.md", "docs/guide.md"),
        ("docs/home.md", "docs/index.md"),
        ("docs/images", "docs/img"),
        ("docs/README.md", "README.md"),
    ]);
    // Not byte for byte: a link that had to climb lost its `./`, and moving back
    // cannot know it was there. What matters is where every link leads.
    assert_eq!(edges(&project.tree()), edges(&before));
}
