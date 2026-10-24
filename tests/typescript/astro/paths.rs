//! Paths that Astro projects write as text: the folder of a content
//! collection, a bundled `<script src>`, a stylesheet imported without `.css`.

use super::{TSCONFIG, astro_project};
use crate::common::project::Project;

const CONFIG: &str = "export const blog = { base: \"./src/Content/blog\", pattern: \"**/*.mdx\" };\nexport const font = [\"src/assets/fonts/a.woff2\"];\n";

fn collection_project() -> Project {
    let project = astro_project();
    project.write("src/Content/blog/post.mdx", "# Post\n");
    project.write("src/assets/fonts/a.woff2", "font");
    project.write("src/content.config.ts", CONFIG);
    project
}

/// The Markdown driver moves a folder of pages and fixes their links; the
/// collection that reads the folder is code, and named it by a string.
#[test]
fn moving_a_content_folder_fixes_the_collection_that_names_it() {
    let project = collection_project();

    let output = project.move_ok(&[("src/Content/blog", "src/Content/articles")]);

    assert!(
        project
            .read("src/content.config.ts")
            .contains("base: \"./src/Content/articles\""),
        "{output}"
    );
    assert!(project.exists("src/Content/articles/post.mdx"));
    assert!(!project.exists("src/Content/blog"));
}

/// A picture or font the code lists in a config object by its project path.
#[test]
fn moving_a_font_fixes_the_config_that_lists_it() {
    let project = collection_project();

    project.move_ok(&[("src/assets/fonts/a.woff2", "src/assets/fonts/latin/a.woff2")]);

    assert!(
        project
            .read("src/content.config.ts")
            .contains("\"src/assets/fonts/latin/a.woff2\"")
    );
    assert!(project.exists("src/assets/fonts/latin/a.woff2"));
}

/// A folder nothing names is moved without starting the engine, so it never
/// depends on the project's code being readable.
#[test]
fn a_content_folder_that_no_code_names_moves_alone() {
    let project = collection_project();
    project.write("src/Content/legal/terms.mdx", "# Terms\n");

    project.move_ok(&[("src/Content/legal", "src/Content/imprint")]);

    assert_eq!(project.read("src/content.config.ts"), CONFIG);
    assert!(project.exists("src/Content/imprint/terms.mdx"));
}

#[test]
fn moving_a_script_fixes_the_script_src_that_bundles_it() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("src/scripts/menu.ts", "console.log(\"menu\");\n");
    project.write(
        "src/pages/index.astro",
        "<script src=\"../scripts/menu.ts\"></script>\n<script is:inline src=\"../scripts/menu.ts\"></script>\n",
    );

    project.move_ok(&[("src/scripts/menu.ts", "src/scripts/ui/menu.ts")]);

    assert_eq!(
        project.read("src/pages/index.astro"),
        "<script src=\"../scripts/ui/menu.ts\"></script>\n<script is:inline src=\"../scripts/menu.ts\"></script>\n"
    );
}

/// Vite adds `.css` to `@import "../Base/main"`; the path stays as written.
#[test]
fn moving_a_stylesheet_fixes_an_import_written_without_css() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("src/styles/base/main.css", ".base { color: red; }\n");
    project.write("src/styles/pages/posts.css", "@import \"../base/main\";\n");

    project.move_ok(&[("src/styles/base/main.css", "src/styles/core/main.css")]);

    assert_eq!(
        project.read("src/styles/pages/posts.css"),
        "@import \"../core/main\";\n"
    );
}

/// A gallery reads a folder of pictures with a glob; the Markdown driver moves
/// the folder, and the engine first points the pattern at the new place.
#[test]
fn moving_a_folder_of_pictures_fixes_the_glob_that_reads_it() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("src/assets/team/ada.webp", "image");
    project.write(
        "src/components/Team.astro",
        "---\nconst photos = import.meta.glob(\"../assets/team/*.webp\", { eager: true });\nconst icons = import.meta.glob(\"/src/icons/*.svg\");\n---\n<div />\n",
    );

    project.move_ok(&[("src/assets/team", "src/assets/people")]);

    assert_eq!(
        project.read("src/components/Team.astro"),
        "---\nconst photos = import.meta.glob(\"../assets/people/*.webp\", { eager: true });\nconst icons = import.meta.glob(\"/src/icons/*.svg\");\n---\n<div />\n"
    );
    assert!(project.exists("src/assets/people/ada.webp"));
    assert!(!project.exists("src/assets/team"));
}

const PAGE: &str = "---\ntitle: Contact\nseo:\n  image:\n    src: ../../assets/open.webp\n    alt: A door\ncanonical: ../about\n---\n\n# Contact\n";

fn page_project() -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("src/assets/open.webp", "image");
    project.write("src/content/en/contact.mdx", PAGE);
    project.write("src/content/en/note.md", PAGE);
    project
}

/// A content collection reads `image()` fields of the frontmatter relative to
/// the page. The Markdown driver does not treat frontmatter as links.
#[test]
fn moving_a_picture_fixes_the_frontmatter_of_the_pages_that_name_it() {
    let project = page_project();

    project.move_ok(&[("src/assets/open.webp", "src/assets/doors/open.webp")]);

    for page in ["src/content/en/contact.mdx", "src/content/en/note.md"] {
        assert_eq!(
            project.read(page),
            PAGE.replace("assets/open.webp", "assets/doors/open.webp")
        );
    }
    assert!(project.exists("src/assets/doors/open.webp"));
}

#[test]
fn moving_a_page_keeps_its_frontmatter_naming_the_same_picture() {
    let project = page_project();

    project.move_ok(&[(
        "src/content/en/contact.mdx",
        "src/content/en/2024/contact.mdx",
    )]);

    assert_eq!(
        project.read("src/content/en/2024/contact.mdx"),
        PAGE.replace("../../assets", "../../../assets")
    );
    assert!(!project.exists("src/content/en/contact.mdx"));
}

#[test]
fn the_dry_run_of_a_picture_move_names_the_pages_with_frontmatter_it_would_edit() {
    let project = page_project();

    let plan = crate::common::dry_run::assert_plan_matches_move(
        &project,
        &[("src/assets/open.webp", "src/assets/doors/open.webp")],
    );

    let edited: Vec<&str> = plan["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert!(edited.contains(&"src/content/en/contact.mdx"), "{plan}");
    assert!(edited.contains(&"src/content/en/note.md"), "{plan}");
}

fn stylesheet_folder(picture: bool) -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write(
        "src/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
    );
    project.write("src/styles/base.css", ".base {}\n");
    project.write(
        "src/styles/pages/posts.css",
        "@import \"../base\";\n.logo { background: url(../../assets/logo.svg); }\n",
    );
    project.write(
        "src/components/Card.astro",
        "---\nimport \"../styles/pages/posts.css\";\n---\n<div />\n",
    );
    if picture {
        project.write(
            "src/styles/pages/bg.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
        );
    }
    project
}

/// A folder of stylesheets is code for the engine: the importers and the
/// paths inside the stylesheets follow it.
#[test]
fn moving_a_folder_of_stylesheets_fixes_importers_and_their_own_paths() {
    for picture in [false, true] {
        let project = stylesheet_folder(picture);

        project.move_ok(&[("src/styles/pages", "src/styles/sections/pages")]);

        assert_eq!(
            project.read("src/styles/sections/pages/posts.css"),
            "@import \"../../base\";\n.logo { background: url(../../../assets/logo.svg); }\n",
            "picture: {picture}"
        );
        assert!(
            project
                .read("src/components/Card.astro")
                .contains("import \"../styles/sections/pages/posts.css\";"),
            "picture: {picture}"
        );
        assert!(!project.exists("src/styles/pages"), "picture: {picture}");
    }
}
