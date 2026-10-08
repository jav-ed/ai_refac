use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::Project;

fn project() -> Project {
    let project = Project::empty();
    project.write(
        "docs/guide.md",
        "# Guide\n\nSee [the logo](../img/logo.png) and [setup](setup.md#top).\n",
    );
    project.write(
        "docs/setup.md",
        "# Setup\n\nBack to the [guide](guide.md).\n",
    );
    project.write("img/logo.png", "not really a png");
    project.write(
        "README.md",
        "[guide](docs/guide.md) and [logo](img/logo.png)\n",
    );
    project
}

#[test]
fn moving_a_page_plans_the_links_to_it_and_the_links_it_holds() {
    let project = project();
    let plan = assert_plan_matches_move(&project, &[("docs/guide.md", "manual/guide.md")]);
    // README (1 link), setup.md (1 link), and the guide itself (2 relative links).
    assert_eq!(plan["edited_files"], 3, "{plan}");
}

#[test]
fn moving_an_asset_plans_the_links_to_it() {
    let project = project();
    let plan = assert_plan_matches_move(&project, &[("img/logo.png", "assets/logo.png")]);
    assert_eq!(plan["edited_files"], 2, "{plan}");
}

#[test]
fn a_folder_of_documents_plans_every_link_across_its_boundary() {
    let project = project();
    assert_plan_matches_move(&project, &[("docs", "handbook/docs")]);
}

#[test]
fn links_to_a_file_another_language_moves_are_part_of_the_plan() {
    let project = project();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
    );
    project.write("src/util.ts", "export const util = 1;\n");
    project.write("NOTES.md", "The code is in [util](src/util.ts).\n");
    let plan = assert_plan_matches_move(&project, &[("src/util.ts", "src/lib/util.ts")]);
    assert_eq!(plan["edited_files"], 1, "{plan}");
}

#[test]
fn a_link_inside_a_moved_page_to_a_moved_source_file_is_counted_once() {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
    );
    project.write("src/util.ts", "export const util = 1;\n");
    project.write("docs/page.md", "The code is in [util](../src/util.ts).\n");
    let plan = assert_plan_matches_move(
        &project,
        &[
            ("src/util.ts", "src/lib/util.ts"),
            ("docs/page.md", "manual/page.md"),
        ],
    );
    assert_eq!(plan["edits"], 1, "one link was rewritten once: {plan}");
}
