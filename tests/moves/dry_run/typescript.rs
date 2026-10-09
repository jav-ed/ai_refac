use crate::common::dry_run::assert_plan_matches_move;
use crate::common::project::{Project, assert_same_tree};

fn project() -> Project {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
    );
    project.write(
        "src/utils/format.ts",
        "import { trim } from \"./trim\";\nexport function fmt(s: string) { return trim(s); }\n",
    );
    project.write(
        "src/utils/trim.ts",
        "export function trim(s: string) { return s.trim(); }\n",
    );
    project.write(
        "src/app.ts",
        "import { fmt } from \"./utils/format\";\nconsole.log(fmt(\"hi\"));\n",
    );
    project.write("src/other.ts", "export const other = 1;\n");
    project
}

#[test]
fn the_plan_of_a_file_move_names_the_importers_and_the_moved_file_itself() {
    let project = project();
    let plan = assert_plan_matches_move(
        &project,
        &[("src/utils/format.ts", "src/helpers/format.ts")],
    );
    // app.ts imports the file; the moved file's own relative import changes too.
    assert_eq!(plan["edited_files"], 2, "{plan}");
    assert_eq!(plan["edits"], 2, "{plan}");
}

#[test]
fn the_plan_of_a_folder_move_leaves_the_files_inside_it_unedited() {
    let project = project();
    let plan = assert_plan_matches_move(&project, &[("src/utils", "src/helpers")]);
    // Only app.ts changes: the files inside keep their relative import of each other.
    assert_eq!(plan["edited_files"], 1, "{plan}");
}

#[test]
fn a_move_nobody_imports_plans_no_edit() {
    let project = project();
    let plan = assert_plan_matches_move(&project, &[("src/other.ts", "src/more/other.ts")]);
    assert_eq!(plan["edits"], 0, "{plan}");
}

#[test]
fn the_text_report_has_the_shape_of_a_real_move() {
    let project = project();
    let output = crate::common::run_cli(&[
        "move",
        "--dry-run",
        "--project-path",
        project.path().to_str().unwrap(),
        "--source-path",
        "src/utils/format.ts",
        "--target-path",
        "src/helpers/format.ts",
    ]);
    let text = crate::common::stdout_text(&output);
    assert!(output.status.success(), "{text}");
    assert!(
        text.starts_with("// Dry run: nothing was changed."),
        "{text}"
    );
    assert!(
        text.contains("src/utils/format.ts -> src/helpers/format.ts"),
        "{text}"
    );
    assert!(text.contains("// src/app.ts (1 edit)"), "{text}");
    assert!(project.exists("src/utils/format.ts"));
}

#[test]
fn a_dry_run_refuses_a_move_whose_new_place_another_module_answers() {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs","moduleResolution":"node"},"include":["src/**/*"]}"#,
    );
    project.write("src/old.ts", "export const old = 1;\n");
    project.write("src/lib.ts", "export const lib = 2;\n");
    project.write(
        "src/main.ts",
        "import { old } from \"./old\";\nimport { lib } from \"./lib\";\nconsole.log(old, lib);\n",
    );
    // main.ts would import the moved module as "./lib", and "./lib" answers
    // with src/lib.ts before it looks into a folder. The real move rolls this
    // back, so the dry run must refuse it too.
    let moves = [("src/old.ts", "src/lib/index.ts")];
    let before = project.tree();

    let dry = project.dry_run_json(&moves);
    let dry_text = crate::common::stderr_text(&dry);
    assert!(!dry.status.success(), "{dry_text}");
    assert!(
        dry_text.contains("Moved module resolves incorrectly"),
        "{dry_text}"
    );
    assert_same_tree(&before, &project.tree());

    let real = project.move_err(&moves);
    assert!(real.contains("Moved module resolves incorrectly"), "{real}");
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_dry_run_checks_imports_that_use_an_alias_against_the_moved_files() {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs","moduleResolution":"node","baseUrl":".","paths":{"@lib/*":["src/lib/*"]}},"include":["src/**/*"]}"#,
    );
    project.write("src/lib/util.ts", "export const util = 1;\n");
    project.write(
        "src/app.ts",
        "import { util } from \"@lib/util\";\nconsole.log(util);\n",
    );
    let plan = assert_plan_matches_move(&project, &[("src/lib/util.ts", "src/lib/tools/util.ts")]);
    assert_eq!(plan["edited_files"], 1, "{plan}");
}
