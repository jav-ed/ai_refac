use crate::common;

// Fixture: tests/fixtures/typescript/rename_project/
//
// util.ts exports `total` (a const), `computeSum`, `Counter` (property `count`,
// method `increment`), `Options`, `Mode`, `Color`, and a default function.
// It also declares an unrelated local `total` inside computeSum (line 3), so
// the name `total` is ambiguous inside that one file.
//
// Usages cover: named import, aliased import, namespace import, `export ... from`
// re-export, `export *`, dynamic import destructuring, shorthand properties, a
// shadowing parameter, string and comment text, a .js file, and a .tsx file.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn rename(project: &Path, file: &str, symbol: &str, new_name: &str, extra: &[&str]) -> Output {
    let mut args = vec![
        "rename",
        "--project-path",
        project.to_str().unwrap(),
        "--file",
        file,
        "--symbol",
        symbol,
        "--new-name",
        new_name,
    ];
    args.extend_from_slice(extra);
    common::run_cli(&args)
}

fn snapshot(project: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(project, project, &mut out);
    out
}

fn assert_succeeded(output: &Output) {
    let stderr = common::stderr_text(output);
    assert!(output.status.success(), "rename should succeed:\n{stderr}");
}

/// A refused rename must fail with a message naming `expected` and leave every byte alone.
fn assert_refused_untouched(
    project: &Path,
    before: &BTreeMap<String, Vec<u8>>,
    output: &Output,
    expected: &str,
) {
    assert!(
        !output.status.success(),
        "rename should fail:\n{}",
        common::stdout_text(output)
    );
    let stderr = common::stderr_text(output);
    assert!(
        stderr.contains(expected),
        "error should mention `{expected}`:\n{stderr}"
    );
    assert_eq!(
        &snapshot(project),
        before,
        "a refused rename must not change any file"
    );
}

async fn assert_typechecks(project: &Path) {
    let executable = refac::drivers::typescript::rename::native_executable()
        .await
        .unwrap();
    let output = Command::new(executable)
        .args(["-p", ".", "--noEmit"])
        .current_dir(project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "renamed project must typecheck:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

// ── exported symbols and every import form ───────────────────────────────────

#[tokio::test]
async fn renames_exported_const_across_every_import_form() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_typechecks(project).await;
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1"],
    ));

    let util = common::read_file(project, "src/lib/util.ts");
    assert!(
        util.contains("export const grandTotal = 10;"),
        "declaration:\n{util}"
    );
    assert!(
        util.contains("const total = items.reduce"),
        "shadowing local untouched:\n{util}"
    );

    let a = common::read_file(project, "src/a.ts");
    assert!(
        a.contains("import { grandTotal, computeSum as sum,"),
        "named import:\n{a}"
    );
    assert!(
        a.contains("export { grandTotal as total } from"),
        "re-export keeps its public name:\n{a}"
    );
    assert!(a.contains("const local = grandTotal + 1;"), "usage:\n{a}");
    assert!(
        a.contains("{ total: grandTotal }"),
        "shorthand property expanded:\n{a}"
    );
    assert!(
        a.contains("util.grandTotal"),
        "namespace member access:\n{a}"
    );
    assert!(
        a.contains("function shadow(total: number) { return total * 2; }"),
        "shadowing parameter untouched:\n{a}"
    );
    assert!(
        a.contains("\"total in string\"") && a.contains("// total in comment"),
        "strings and comments untouched:\n{a}"
    );

    let b = common::read_file(project, "src/b.ts");
    assert!(
        b.contains("const { grandTotal: total } = await import"),
        "dynamic import destructuring:\n{b}"
    );
    let d = common::read_file(project, "src/d.js");
    assert!(
        d.contains("import { grandTotal } from") && d.contains("grandTotal + 1"),
        "javascript file:\n{d}"
    );
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_class_property_from_a_usage_site() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/a.ts", "count", "value", &[]));

    assert!(common::read_file(project, "src/lib/util.ts").contains("    value = 0;"));
    assert!(common::read_file(project, "src/lib/util.ts").contains("this.value++"));
    assert!(common::read_file(project, "src/a.ts").contains("c.value"));
    assert!(common::read_file(project, "src/c.tsx").contains("props.counter.value"));
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_default_exported_function_and_its_importers() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "defaultFn",
        "mainFn",
        &[],
    ));

    assert!(
        common::read_file(project, "src/lib/util.ts").contains("export default function mainFn()")
    );
    let a = common::read_file(project, "src/a.ts");
    assert!(
        a.contains("import mainFn from") && a.contains("mainFn()"),
        "default import follows:\n{a}"
    );
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_local_export_without_changing_the_public_name() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/a.ts", "local", "localTotal", &[]));

    let a = common::read_file(project, "src/a.ts");
    assert!(a.contains("const localTotal = total + 1;"), "{a}");
    assert!(
        a.contains("export { localTotal as local,"),
        "importers of `local` keep working:\n{a}"
    );
    assert_typechecks(project).await;
}

// ── choosing the symbol ──────────────────────────────────────────────────────

#[test]
fn ambiguous_name_lists_candidates_and_changes_nothing() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/lib/util.ts", "total", "grandTotal", &[]);

    assert_refused_untouched(project, &before, &output, "several different symbols");
    let stderr = common::stderr_text(&output);
    assert!(
        stderr.contains("1:14") && stderr.contains("3:11"),
        "both candidates listed:\n{stderr}"
    );
}

#[tokio::test]
async fn line_selects_the_shadowing_local_only() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "total",
        "sumTotal",
        &["--line", "3"],
    ));

    let util = common::read_file(project, "src/lib/util.ts");
    assert!(
        util.contains("export const total = 10;"),
        "module-level total untouched:\n{util}"
    );
    assert!(
        util.contains("const sumTotal = items.reduce") && util.contains("return sumTotal;"),
        "{util}"
    );
    assert!(
        common::read_file(project, "src/a.ts").contains("import { total,"),
        "importers untouched"
    );
    assert_typechecks(project).await;
}

#[test]
fn column_narrows_a_line_with_two_symbols() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    // Column 8 is not an occurrence of `total` on line 3, so nothing matches.
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "x",
        &["--line", "3", "--column", "8"],
    );
    assert_refused_untouched(
        project,
        &before,
        &output,
        "does not appear as an identifier at 3:8",
    );
}

// ── safety: collisions, shadowing, refusals ──────────────────────────────────

#[test]
fn name_collision_is_refused_before_any_write() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "computeSum",
        &["--line", "1"],
    );
    assert_refused_untouched(project, &before, &output, "not faithful");
}

#[test]
fn shadow_capture_that_still_typechecks_is_refused() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    // `inner` -> `outer` compiles (both numbers) but makes `outer` mean the local.
    let output = rename(project, "src/shadow.ts", "inner", "outer", &[]);
    assert_refused_untouched(project, &before, &output, "not faithful");
}

#[test]
fn standard_library_symbols_are_refused() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/lib/util.ts", "reduce", "fold", &[]);
    assert_refused_untouched(project, &before, &output, "standard TypeScript library");
}

#[test]
fn text_inside_strings_is_not_a_symbol() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/a.ts", "string", "text", &["--line", "13"]);
    assert_refused_untouched(project, &before, &output, "Cannot rename");
}

#[test]
fn invalid_requests_are_rejected_early() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    for (new_name, expected) in [
        ("class", "reserved word"),
        ("2fast", "not a valid identifier"),
        ("total", "equals the current name"),
    ] {
        let output = rename(
            project,
            "src/lib/util.ts",
            "total",
            new_name,
            &["--line", "1"],
        );
        assert_refused_untouched(project, &before, &output, expected);
    }
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "x",
        &["--column", "14"],
    );
    assert!(!output.status.success(), "--column needs --line");
    let output = rename(project, "tsconfig.json", "compilerOptions", "x", &[]);
    assert_refused_untouched(
        project,
        &before,
        &output,
        "Python (.py) and Dart (.dart) files",
    );
}

// ── output modes ─────────────────────────────────────────────────────────────

#[test]
fn dry_run_reports_json_and_writes_nothing() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1", "--dry-run", "--json"],
    );
    assert_succeeded(&output);
    assert_eq!(snapshot(project), before, "dry run must not write");

    let json: serde_json::Value = serde_json::from_str(&common::stdout_text(&output)).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["operation"], "rename");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["edits"], 9);
    assert_eq!(json["edited_files"], 4);
}

#[test]
fn dry_run_text_says_nothing_changed_and_how_to_apply_it() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1", "--dry-run"],
    );
    assert_succeeded(&output);
    assert_eq!(snapshot(project), before, "dry run must not write");

    let text = common::stdout_text(&output);
    assert!(
        text.contains("// Dry run: nothing was changed. Planned and verified rename:"),
        "{text}"
    );
    assert!(text.contains("total -> grandTotal"), "{text}");
    assert!(
        text.contains("// Run the same command without --dry-run to write these edits."),
        "{text}"
    );
}

// ── project and file conditions ──────────────────────────────────────────────

#[test]
fn utf8_columns_and_multibyte_text_are_handled() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/unicode.ts", "amount", "sum", &[]));
    let text = common::read_file(project, "src/unicode.ts");
    assert!(
        text.contains("const café = \"😀é\";"),
        "multibyte text intact:\n{text}"
    );
    assert!(
        text.contains("export const sum = café.length;") && text.contains("sum * 2"),
        "{text}"
    );
}

#[test]
fn bom_and_crlf_line_endings_are_preserved() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let original = "\u{FEFF}export const amount = 1;\r\nexport const twice = amount * 2;\r\n";
    fs::write(project.join("src/windows.ts"), original).unwrap();
    assert_succeeded(&rename(
        project,
        "src/windows.ts",
        "amount",
        "quantity",
        &[],
    ));
    let bytes = fs::read(project.join("src/windows.ts")).unwrap();
    let expected = "\u{FEFF}export const quantity = 1;\r\nexport const twice = quantity * 2;\r\n";
    assert_eq!(String::from_utf8(bytes).unwrap(), expected);
}

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
