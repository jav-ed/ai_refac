//! Moves of single files in a small documentation site
//! (`tests/fixtures/markdown/site`): a move, a rename, and the files that must
//! not change. Each test checks the exact text of what changes, that every
//! Markdown file still links to the same files (second reader in
//! `common/links.rs`), and that moving back restores every byte.

mod common;

use common::project::{assert_same_tree, changed, text};
use common::site::{assert_links_follow, edited, site, sorted};

#[test]
fn moving_a_file_updates_the_links_to_it_and_the_links_inside_it() {
    let project = site();
    let before = project.tree();

    let output = project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);
    let after = project.tree();

    assert!(
        output.contains("Checked 9 Markdown files; updated 13 links in 9 files."),
        "{output}"
    );
    assert_eq!(
        changed(&before, &after),
        sorted(vec![
            ".github/CONTRIBUTING.md",
            "CHANGELOG.md",
            "README.md",
            "docs/My Notes.md",
            "docs/api/endpoints.md",
            "docs/api/overview.md",
            "docs/guide.md",
            "docs/index.md",
            "docs/manual/guide.md",
            "docs/page.mdx",
        ])
    );

    let guide = "(docs/guide.md";
    assert_eq!(
        text(&after, "README.md"),
        edited(
            &before,
            "README.md",
            &[(
                "[guide](docs/guide.md#install)",
                "[guide](docs/manual/guide.md#install)"
            )]
        ),
        "the code span and the web address that mention {guide} stay"
    );
    assert_eq!(
        text(&after, "CHANGELOG.md"),
        edited(
            &before,
            "CHANGELOG.md",
            &[("](docs/guide.md)", "](docs/manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/index.md"),
        edited(
            &before,
            "docs/index.md",
            &[("[Guide](guide.md)", "[Guide](manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/My Notes.md"),
        edited(
            &before,
            "docs/My Notes.md",
            &[("](guide.md)", "](manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/api/overview.md"),
        edited(
            &before,
            "docs/api/overview.md",
            &[("](../guide.md)", "](../manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/api/endpoints.md"),
        edited(
            &before,
            "docs/api/endpoints.md",
            &[("](../guide.md)", "](../manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/page.mdx"),
        edited(
            &before,
            "docs/page.mdx",
            &[("](./guide.md)", "](./manual/guide.md)")]
        ),
        "the author's ./ stays"
    );
    assert_eq!(
        text(&after, ".github/CONTRIBUTING.md"),
        edited(
            &before,
            ".github/CONTRIBUTING.md",
            &[("(../docs/guide.md)", "(../docs/manual/guide.md)")]
        )
    );
    assert_eq!(
        text(&after, "docs/manual/guide.md"),
        edited(
            &before,
            "docs/guide.md",
            &[
                ("[API](api/overview.md)", "[API](../api/overview.md)"),
                (
                    "[endpoints](api/endpoints.md#list)",
                    "[endpoints](../api/endpoints.md#list)"
                ),
                ("[index](index.md)", "[index](../index.md)"),
                ("[readme](../README.md)", "[readme](../../README.md)"),
                (
                    "![diagram](img/diagram.svg)",
                    "![diagram](../img/diagram.svg)"
                ),
            ]
        ),
        "a link that climbs gets no ./, web addresses and #anchors are untouched"
    );

    assert_links_follow(&before, &after, &|path| {
        if path == "docs/guide.md" {
            "docs/manual/guide.md".to_string()
        } else {
            path.to_string()
        }
    });
}

#[test]
fn moving_a_file_and_moving_it_back_restores_every_byte() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);
    project.move_ok(&[("docs/manual/guide.md", "docs/guide.md")]);

    assert_same_tree(&before, &project.tree());
}

#[test]
fn renaming_a_file_in_place_changes_only_the_links_to_it() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[("docs/guide.md", "docs/handbook.md")]);
    let after = project.tree();

    // Same folder, so nothing inside the file moves.
    assert_eq!(
        text(&after, "docs/handbook.md"),
        text(&before, "docs/guide.md")
    );
    assert_eq!(
        text(&after, "docs/index.md"),
        edited(
            &before,
            "docs/index.md",
            &[("[Guide](guide.md)", "[Guide](handbook.md)")]
        )
    );
    assert_eq!(
        text(&after, "README.md"),
        edited(
            &before,
            "README.md",
            &[("(docs/guide.md#install)", "(docs/handbook.md#install)")]
        )
    );
    assert_links_follow(&before, &after, &|path| {
        if path == "docs/guide.md" {
            "docs/handbook.md".to_string()
        } else {
            path.to_string()
        }
    });

    project.move_ok(&[("docs/handbook.md", "docs/guide.md")]);
    assert_same_tree(&before, &project.tree());
}

#[test]
fn the_files_that_were_not_touched_are_byte_for_byte_the_same() {
    let project = site();
    let before = project.tree();

    project.move_ok(&[("docs/api/endpoints.md", "docs/api/list.md")]);
    let after = project.tree();

    for path in [
        "docs/index.md",
        "docs/img/diagram.svg",
        "docs/img/logo.png",
        "docs/page.mdx",
    ] {
        assert_eq!(after.get(path), before.get(path), "{path}");
    }
    assert_eq!(
        changed(&before, &after),
        sorted(vec![
            "CHANGELOG.md",
            "docs/api/endpoints.md",
            "docs/api/list.md",
            "docs/api/overview.md",
            "docs/guide.md",
        ])
    );
}
