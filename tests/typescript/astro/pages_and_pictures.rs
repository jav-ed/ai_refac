//! Moving pages and pictures that code imports: the Markdown driver moves
//! them, the TypeScript engine fixes the code first.

use super::{TSCONFIG, astro_project};
use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::Project;

fn picture_project() -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write(
        "src/assets/hero.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
    );
    project.write(
        "src/components/Hero.astro",
        "---\nimport art from \"../assets/hero.svg\";\n---\n<img src={art.src} />\n",
    );
    project.write(
        "src/lib/art.ts",
        "import hero from \"../assets/hero.svg?url\";\nexport default hero;\n",
    );
    project.write("README.md", "![hero](src/assets/hero.svg)\n");
    project
}

/// The Markdown driver used to move the picture and leave both importers
/// pointing at the old place, with the build the only one to notice.
#[test]
fn moving_a_picture_that_code_imports_fixes_the_imports_and_the_markdown_links() {
    let project = picture_project();

    let output = project.move_ok(&[("src/assets/hero.svg", "src/images/hero.svg")]);

    // The Markdown driver moves the picture; the engine fixed the code first.
    assert!(output.contains("Markdown results"), "{output}");
    assert!(
        output.contains("Updated the imports in 2 code file(s)"),
        "{output}"
    );
    assert!(
        project
            .read("src/components/Hero.astro")
            .contains("\"../images/hero.svg\"")
    );
    assert!(
        project
            .read("src/lib/art.ts")
            .contains("\"../images/hero.svg?url\"")
    );
    assert_eq!(project.read("README.md"), "![hero](src/images/hero.svg)\n");
    assert!(project.exists("src/images/hero.svg"));
}

#[test]
fn the_dry_run_of_a_picture_move_names_the_importers_it_would_edit() {
    let project = picture_project();

    let plan =
        assert_plan_matches_move(&project, &[("src/assets/hero.svg", "src/images/hero.svg")]);

    let edited: Vec<&str> = plan["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert!(edited.contains(&"src/components/Hero.astro"), "{plan}");
    assert!(edited.contains(&"src/lib/art.ts"), "{plan}");
    assert!(edited.contains(&"README.md"), "{plan}");
}

/// Pictures that no code uses must not start the engine: a component refac
/// cannot read has nothing to do with a documentation move.
#[test]
fn moving_a_picture_that_no_code_uses_stays_a_markdown_move() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("docs/guide.md", "![shot](shot.png)\n");
    project.write_bytes("docs/shot.png", b"png");
    project.write("src/pages/broken.astro", "---\nconst = ;\n---\n<p></p>\n");

    let output = project.move_ok(&[("docs/shot.png", "docs/img/shot.png")]);

    assert!(output.contains("Markdown results"), "{output}");
    assert!(!output.contains("Typescript results"), "{output}");
    assert_eq!(project.read("docs/guide.md"), "![shot](img/shot.png)\n");
}

/// The relative paths in an `.mdx` page's own `import` lines depend on its
/// folder; the page's Markdown links and the pages that import it follow too.
#[test]
fn moving_an_mdx_page_rewrites_its_own_imports_links_and_importers() {
    let project = astro_project();
    project.write("src/content/pic.png", "png");
    project.write(
        "src/content/post.mdx",
        "import Card from \"../components/Card.astro\";\nimport { formatDate } from \"../lib/format\";\n\n![pic](./pic.png)\n\n<Card />\n",
    );
    project.write(
        "src/pages/blog.astro",
        "---\nimport Post from \"../content/post.mdx\";\n---\n<Post />\n",
    );

    let output = project.move_ok(&[("src/content/post.mdx", "src/content/2026/post.mdx")]);

    assert!(output.contains("Markdown results"), "{output}");
    assert_eq!(
        project.read("src/content/2026/post.mdx"),
        "import Card from \"../../components/Card.astro\";\nimport { formatDate } from \"../../lib/format\";\n\n![pic](../pic.png)\n\n<Card />\n"
    );
    assert!(
        project
            .read("src/pages/blog.astro")
            .contains("\"../content/2026/post.mdx\"")
    );
}

#[test]
fn the_dry_run_of_an_mdx_page_move_matches_the_move() {
    let project = astro_project();
    project.write("src/content/pic.png", "png");
    project.write(
        "src/content/post.mdx",
        "import Card from \"../components/Card.astro\";\n\n![pic](./pic.png)\n\n<Card />\n",
    );
    project.write(
        "src/pages/blog.astro",
        "---\nimport Post from \"../content/post.mdx\";\n---\n<Post />\n",
    );

    assert_plan_matches_move(
        &project,
        &[("src/content/post.mdx", "src/content/2026/post.mdx")],
    );
}

fn styled_project() -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write(
        "src/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
    );
    project.write(
        "src/styles/global.css",
        ".logo { background: url(../assets/logo.svg); }\n",
    );
    project.write(
        "src/components/Card.astro",
        "---\nimport \"../styles/global.css\";\n---\n<div class=\"card\" />\n<style>\n  .card { background: url(\"../assets/logo.svg\") no-repeat; }\n</style>\n",
    );
    project
}

/// A build passes with a stale `url()`: it only warns that the file stays
/// unresolved, and the page then asks for a picture that is not there.
#[test]
fn moving_a_picture_fixes_the_urls_in_style_blocks_and_stylesheets() {
    let project = styled_project();

    let output = project.move_ok(&[("src/assets/logo.svg", "src/assets/brand/logo.svg")]);

    assert!(output.contains("Markdown results"), "{output}");
    assert!(
        project
            .read("src/components/Card.astro")
            .contains("url(\"../assets/brand/logo.svg\")")
    );
    assert_eq!(
        project.read("src/styles/global.css"),
        ".logo { background: url(../assets/brand/logo.svg); }\n"
    );
}

#[test]
fn moving_a_stylesheet_fixes_its_importers_and_its_own_urls() {
    let project = styled_project();

    let output = project.move_ok(&[("src/styles/global.css", "src/styles/core/global.css")]);

    assert!(output.contains("Typescript results"), "{output}");
    assert!(
        project
            .read("src/components/Card.astro")
            .contains("import \"../styles/core/global.css\";")
    );
    assert_eq!(
        project.read("src/styles/core/global.css"),
        ".logo { background: url(../../assets/logo.svg); }\n"
    );
}

#[test]
fn the_dry_run_of_a_picture_move_names_the_stylesheets_it_would_edit() {
    let project = styled_project();

    let plan = assert_plan_matches_move(
        &project,
        &[("src/assets/logo.svg", "src/assets/brand/logo.svg")],
    );

    let edited: Vec<&str> = plan["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert!(edited.contains(&"src/styles/global.css"), "{plan}");
    assert!(edited.contains(&"src/components/Card.astro"), "{plan}");
}
