//! Projects the engine cannot rename in: legacy config, no tsconfig, files outside it, usages elsewhere.

use super::{assert_refused_untouched, rename, snapshot};
use crate::common;

use std::fs;

#[test]
fn legacy_base_url_config_is_rejected_with_the_engine_diagnostic() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","baseUrl":".","jsx":"preserve","allowJs":true},"include":["src/**/*"]}"#,
    )
    .unwrap();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1"],
    );
    assert_refused_untouched(project, &before, &output, "TS5102");
}

#[test]
fn project_without_tsconfig_is_rejected() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    fs::remove_file(project.join("tsconfig.json")).unwrap();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1"],
    );
    assert_refused_untouched(project, &before, &output, "No tsconfig.json");
}

#[test]
fn file_outside_the_tsconfig_is_rejected() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    fs::create_dir_all(project.join("scripts")).unwrap();
    fs::write(
        project.join("scripts/tool.ts"),
        "export const helper = 1;\n",
    )
    .unwrap();
    let before = snapshot(project);
    let output = rename(project, "scripts/tool.ts", "helper", "assistant", &[]);
    assert_refused_untouched(project, &before, &output, "is not part of the tsconfig");
}

#[test]
fn usages_in_a_discoverable_referencing_project_are_refused_not_edited() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for dir in ["lib", "app"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    let options =
        r#""module":"esnext","moduleResolution":"bundler","target":"es2022","strict":true"#;
    fs::write(root.join("lib/tsconfig.json"), format!(r#"{{"compilerOptions":{{{options},"composite":true,"outDir":"dist"}},"include":["*.ts"]}}"#)).unwrap();
    fs::write(root.join("lib/util.ts"), "export const shared = 1;\n").unwrap();
    fs::write(root.join("app/tsconfig.json"), format!(r#"{{"compilerOptions":{{{options},"noEmit":true}},"include":["*.ts"],"references":[{{"path":"../lib"}}]}}"#)).unwrap();
    fs::write(
        root.join("app/main.ts"),
        "import { shared } from \"../lib/util\";\nexport const x = shared + 1;\n",
    )
    .unwrap();
    // A solution-style root tsconfig is how the engine discovers that `app`
    // uses `lib`. Without it the engine cannot see the usage at all, so the
    // owning tsconfig must include every caller (same rule as file moves).
    fs::write(
        root.join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"./lib"},{"path":"./app"}]}"#,
    )
    .unwrap();

    let before = snapshot(root);
    // From lib, the usage in app lies outside the project: refuse, change nothing.
    let output = rename(&root.join("lib"), "util.ts", "shared", "common", &[]);
    assert_refused_untouched(root, &before, &output, "outside the project");
}
