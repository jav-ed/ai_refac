//! Moving scripts and components that `.astro` and `.mdx` files import.

use super::{TSCONFIG, astro_project};
use crate::common::project::{Project, assert_same_tree};

#[test]
fn moving_a_script_fixes_the_astro_and_mdx_files_that_import_it() {
    let project = astro_project();

    let output = project.move_ok(&[("src/lib/format.ts", "src/lib/text/format.ts")]);

    assert!(
        output.contains("1 TypeScript/JavaScript source file moved"),
        "{output}"
    );
    assert!(
        project
            .read("src/components/Card.astro")
            .contains("\"../lib/text/format\"")
    );
    assert!(
        project
            .read("src/pages/index.astro")
            .contains("\"../lib/text/format\"")
    );
    assert!(
        project
            .read("src/content/post.mdx")
            .contains("\"../lib/text/format\"")
    );
}

#[test]
fn moving_a_component_keeps_its_importers_working_alias_included() {
    let project = astro_project();

    project.move_ok(&[("src/components/Card.astro", "src/components/ui/Card.astro")]);

    // The component's own import follows its new place.
    assert!(
        project
            .read("src/components/ui/Card.astro")
            .contains("\"../../lib/format\"")
    );
    // An alias import stays an alias; a relative one stays relative.
    assert!(
        project
            .read("src/pages/index.astro")
            .contains("\"@components/ui/Card.astro\"")
    );
    assert!(
        project
            .read("src/content/post.mdx")
            .contains("\"../components/ui/Card.astro\"")
    );
    assert!(!project.exists("src/components/Card.astro"));
}

/// A component folder with an image used to be moved as a "document folder",
/// reported as a success, and left its importer broken.
#[test]
fn a_component_folder_with_an_image_moves_as_code() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write(
        "src/components/hero/hero.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
    );
    project.write(
        "src/components/hero/Hero.astro",
        "---\nimport art from \"./hero.svg\";\n---\n<img src={art.src} />\n",
    );
    project.write(
        "src/pages/index.astro",
        "---\nimport Hero from \"../components/hero/Hero.astro\";\n---\n<Hero />\n",
    );

    let output = project.move_ok(&[("src/components/hero", "src/sections/hero")]);

    // The old routing answered "Markdown results" here.
    assert!(output.contains("Typescript results"), "{output}");
    assert!(!output.contains("Markdown results"), "{output}");
    assert_eq!(
        project.read("src/pages/index.astro"),
        "---\nimport Hero from \"../sections/hero/Hero.astro\";\n---\n<Hero />\n"
    );
    assert!(project.exists("src/sections/hero/hero.svg"));
}

/// A file refac cannot read must stop the move before anything is written: a
/// skipped file would keep pointing at the old path without a word.
#[test]
fn an_astro_file_that_cannot_be_parsed_stops_the_move_and_changes_nothing() {
    let project = astro_project();
    project.write("src/pages/broken.astro", "---\nconst = ;\n---\n<p></p>\n");
    let before = project.tree();

    let error = project.move_err(&[("src/lib/format.ts", "src/lib/text/format.ts")]);

    assert!(error.contains("Cannot parse"), "{error}");
    assert!(error.contains("broken.astro"), "{error}");
    assert_same_tree(&before, &project.tree());
}

/// The paths in a `<style>` block are relative to the component like imports
/// are; they used to stay behind and fail only in the browser.
#[test]
fn moving_a_component_fixes_the_urls_in_its_style_block() {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write("src/assets/bg.png", "png");
    project.write(
        "src/components/Banner.astro",
        "<section />\n<style>\n  section { background: url(../assets/bg.png); }\n</style>\n",
    );

    project.move_ok(&[(
        "src/components/Banner.astro",
        "src/components/ui/Banner.astro",
    )]);

    assert!(
        project
            .read("src/components/ui/Banner.astro")
            .contains("url(../../assets/bg.png)")
    );
}
