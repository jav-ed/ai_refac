//! The corpus from `Project_Manag/Docs/Investigation/markdown_Options.md`: 13
//! shapes in which a path appears in a README, moved three ways. The same
//! corpus was run through docmv, markmv, and mdref; each missed or damaged some
//! of it. This pins the exact text `refac` produces.

mod common;

use common::project::Project;

const README: &str = "\
# Corpus

1. Inline [guide](docs/guide.md#install) and image ![logo](docs/img/logo.png).
2. HTML <img src=\"docs/img/logo.png\" width=\"10\"> and <a href=\"docs/guide.md\">html link</a>.
3. Multi-line definition: see [multi].
4. Directory [docs](docs/) and [dot-slash](./docs/guide.md).
5. Spaces [a](<docs/My Notes.md>) and escaped [b](docs/My%20Notes.md).
6. Code `[c](docs/guide.md)` and a fence:

```md
[d](docs/guide.md)
```

7. Query [q](docs/img/logo.png?raw=1).
8. Table:

| a | b |
|---|---|
| [x](docs/guide.md) | text |

9. Autolink <https://example.com/docs/guide.md> and web [w](https://example.com/docs/guide.md).
10. Link text over
two lines [text
continues](docs/guide.md).
11. Reference [ref-style][r1] and [collapsed][] and [shortcut].
12. srcset <img srcset=\"docs/img/logo.png 1x, docs/img/logo.png 2x\" src=\"docs/img/logo.png\">
13. Unicode [ü](docs/Übersicht.md).

[multi]:
   docs/guide.md
[r1]: docs/guide.md \"Title\"
[collapsed]: docs/guide.md
[shortcut]: ./docs/guide.md
";

const GUIDE: &str = "\
# Guide
Back to [readme](../README.md), the ![logo](img/logo.png) and [notes](My%20Notes.md).
";

fn corpus() -> Project {
    let project = Project::empty();
    project.write("README.md", README);
    project.write("docs/guide.md", GUIDE);
    project.write("docs/My Notes.md", "# Notes\n");
    project.write("docs/Übersicht.md", "# Übersicht\n");
    project.write_bytes("docs/img/logo.png", b"PNG");
    project
}

/// `README` with each pair applied; a pair that matches nothing is a mistake.
fn readme_with(pairs: &[(&str, &str)]) -> String {
    let mut text = README.to_string();
    for (from, to) in pairs {
        assert!(text.contains(from), "the corpus has no {from:?}");
        text = text.replace(from, to);
    }
    text
}

#[test]
fn moving_the_guide_updates_every_shape_that_points_at_it() {
    let project = corpus();

    project.move_ok(&[("docs/guide.md", "docs/manual/guide.md")]);

    assert_eq!(
        project.read("README.md"),
        readme_with(&[
            (
                "[guide](docs/guide.md#install)",
                "[guide](docs/manual/guide.md#install)"
            ),
            (
                "<a href=\"docs/guide.md\">",
                "<a href=\"docs/manual/guide.md\">"
            ),
            (
                "[dot-slash](./docs/guide.md)",
                "[dot-slash](./docs/manual/guide.md)"
            ),
            ("| [x](docs/guide.md) |", "| [x](docs/manual/guide.md) |"),
            (
                "continues](docs/guide.md)",
                "continues](docs/manual/guide.md)"
            ),
            (
                "[multi]:\n   docs/guide.md",
                "[multi]:\n   docs/manual/guide.md"
            ),
            ("[r1]: docs/guide.md", "[r1]: docs/manual/guide.md"),
            (
                "[collapsed]: docs/guide.md",
                "[collapsed]: docs/manual/guide.md"
            ),
            (
                "[shortcut]: ./docs/guide.md",
                "[shortcut]: ./docs/manual/guide.md"
            ),
        ])
    );
    assert_eq!(
        project.read("docs/manual/guide.md"),
        "# Guide\nBack to [readme](../../README.md), the ![logo](../img/logo.png) and [notes](../My%20Notes.md).\n"
    );
}

#[test]
fn moving_the_folder_updates_every_shape_and_leaves_the_inside_alone() {
    let project = corpus();

    project.move_ok(&[("docs", "documentation")]);

    assert_eq!(
        project.read("README.md"),
        readme_with(&[
            (
                "[guide](docs/guide.md#install) and image ![logo](docs/img/logo.png)",
                "[guide](documentation/guide.md#install) and image ![logo](documentation/img/logo.png)"
            ),
            (
                "<img src=\"docs/img/logo.png\" width",
                "<img src=\"documentation/img/logo.png\" width"
            ),
            (
                "<a href=\"docs/guide.md\">",
                "<a href=\"documentation/guide.md\">"
            ),
            ("[docs](docs/)", "[docs](documentation/)"),
            (
                "[dot-slash](./docs/guide.md)",
                "[dot-slash](./documentation/guide.md)"
            ),
            (
                "[a](<docs/My Notes.md>)",
                "[a](<documentation/My Notes.md>)"
            ),
            (
                "[b](docs/My%20Notes.md)",
                "[b](documentation/My%20Notes.md)"
            ),
            (
                "[q](docs/img/logo.png?raw=1)",
                "[q](documentation/img/logo.png?raw=1)"
            ),
            ("| [x](docs/guide.md) |", "| [x](documentation/guide.md) |"),
            (
                "continues](docs/guide.md)",
                "continues](documentation/guide.md)"
            ),
            (
                "srcset=\"docs/img/logo.png 1x, docs/img/logo.png 2x\" src=\"docs/img/logo.png\"",
                "srcset=\"documentation/img/logo.png 1x, documentation/img/logo.png 2x\" src=\"documentation/img/logo.png\""
            ),
            ("[ü](docs/Übersicht.md)", "[ü](documentation/Übersicht.md)"),
            (
                "[multi]:\n   docs/guide.md",
                "[multi]:\n   documentation/guide.md"
            ),
            ("[r1]: docs/guide.md", "[r1]: documentation/guide.md"),
            (
                "[collapsed]: docs/guide.md",
                "[collapsed]: documentation/guide.md"
            ),
            (
                "[shortcut]: ./docs/guide.md",
                "[shortcut]: ./documentation/guide.md"
            ),
        ])
    );
    // Everything it links to moved with it, so not a byte changes.
    assert_eq!(project.read("documentation/guide.md"), GUIDE);
}

#[test]
fn moving_the_image_updates_markdown_html_query_and_srcset() {
    let project = corpus();

    project.move_ok(&[("docs/img/logo.png", "assets/logo.png")]);

    assert_eq!(
        project.read("README.md"),
        readme_with(&[
            ("![logo](docs/img/logo.png)", "![logo](assets/logo.png)"),
            (
                "<img src=\"docs/img/logo.png\" width",
                "<img src=\"assets/logo.png\" width"
            ),
            ("[q](docs/img/logo.png?raw=1)", "[q](assets/logo.png?raw=1)"),
            (
                "srcset=\"docs/img/logo.png 1x, docs/img/logo.png 2x\" src=\"docs/img/logo.png\"",
                "srcset=\"assets/logo.png 1x, assets/logo.png 2x\" src=\"assets/logo.png\""
            ),
        ])
    );
    assert_eq!(
        project.read("docs/guide.md"),
        GUIDE.replace("![logo](img/logo.png)", "![logo](../assets/logo.png)")
    );
}
