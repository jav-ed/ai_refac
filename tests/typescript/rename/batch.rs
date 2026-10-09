//! Several renames in one engine session (`refac rename --batch`) on the
//! rename fixture: all of them are written against the files the one before
//! left, a failing one takes the earlier ones back, and a dry run plans
//! exactly what the real batch then does.

use super::{assert_typechecks, snapshot};
use crate::common;
use std::path::Path;
use std::process::Output;

const UTIL: &str = "src/lib/util.ts";

/// `total` is declared twice in util.ts (the export and a local of
/// `computeSum`), so the batch names line 1; `Tally` is the name the second
/// rename gave `Counter`.
const BATCH: &str = r#"[
  {"file": "src/lib/util.ts", "symbol": "computeSum", "new_name": "addAll"},
  {"file": "src/lib/util.ts", "symbol": "Counter", "new_name": "Tally"},
  {"file": "src/lib/util.ts", "symbol": "total", "new_name": "grandTotal", "line": 1},
  {"file": "src/lib/util.ts", "symbol": "Tally", "new_name": "Counter2"}
]"#;

fn run_batch(project: &Path, batch: &str, extra: &[&str]) -> Output {
    let file = project.join("batch.json");
    std::fs::write(&file, batch).unwrap();
    let mut args = vec![
        "rename",
        "--project-path",
        project.to_str().unwrap(),
        "--batch",
        file.to_str().unwrap(),
        "--json",
    ];
    args.extend_from_slice(extra);
    let output = common::run_cli(&args);
    std::fs::remove_file(&file).unwrap();
    output
}

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("not JSON ({error}): {}", common::stdout_text(output)))
}

#[tokio::test]
async fn a_batch_renames_in_order_and_the_project_still_typechecks() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();

    let output = run_batch(project, BATCH, &[]);

    assert!(output.status.success(), "{}", common::stderr_text(&output));
    let result = json(&output);
    assert_eq!(result["operation"], "rename-batch");
    assert_eq!(result["renames"].as_array().unwrap().len(), 4);
    let util = std::fs::read_to_string(project.join(UTIL)).unwrap();
    assert!(util.contains("export function addAll("), "{util}");
    assert!(util.contains("export class Counter2 {"), "{util}");
    assert!(util.contains("export const grandTotal = 10;"), "{util}");
    let a = std::fs::read_to_string(project.join("src/a.ts")).unwrap();
    assert!(a.contains("new Counter2()"), "{a}");
    assert!(a.contains("util.addAll([3])"), "{a}");
    assert_typechecks(project).await;
}

#[test]
fn a_dry_run_plans_what_the_real_batch_does_and_writes_nothing() {
    let planned = common::setup_fixture("typescript/rename_project");
    let real = common::setup_fixture("typescript/rename_project");
    let before = snapshot(planned.path());

    let dry = run_batch(planned.path(), BATCH, &["--dry-run"]);
    assert!(dry.status.success(), "{}", common::stderr_text(&dry));
    assert_eq!(snapshot(planned.path()), before, "a dry run writes nothing");
    let done = run_batch(real.path(), BATCH, &[]);
    assert!(done.status.success(), "{}", common::stderr_text(&done));

    // The fourth rename names the symbol by the name the second gave it, so a
    // dry run that planned every rename against the files on disk would fail.
    let (dry, done) = (json(&dry), json(&done));
    assert_eq!(dry["dry_run"], true);
    for key in ["renames", "edits", "edited_files"] {
        let strip = |value: &serde_json::Value| value[key].to_string().replace("true", "false");
        assert_eq!(strip(&dry), strip(&done), "`{key}` of plan and batch");
    }
}

#[test]
fn a_failing_rename_takes_the_written_ones_back() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    // The third rename names a symbol util.ts does not contain.
    let batch = BATCH.replace(
        r#"{"file": "src/lib/util.ts", "symbol": "total", "new_name": "grandTotal", "line": 1}"#,
        r#"{"file": "src/lib/util.ts", "symbol": "nothing_here", "new_name": "grandTotal"}"#,
    );

    let output = run_batch(project, &batch, &[]);

    assert!(!output.status.success());
    let message = common::stderr_text(&output);
    assert!(
        message.contains("Rename 3 of 4 (nothing_here -> grandTotal in src/lib/util.ts)"),
        "{message}"
    );
    assert!(
        message.contains("the 2 earlier rename(s) were undone, so nothing was changed"),
        "{message}"
    );
    assert_eq!(snapshot(project), before);
}

#[test]
fn a_dry_run_refuses_what_the_real_batch_refuses() {
    // Both renames give `Counter` a new name; the second names it by the old
    // one, which the first took away.
    let batch = r#"[
      {"file": "src/lib/util.ts", "symbol": "Counter", "new_name": "Tally"},
      {"file": "src/lib/util.ts", "symbol": "Counter", "new_name": "Count"}
    ]"#;
    for extra in [&["--dry-run"][..], &[][..]] {
        let temp = common::setup_fixture("typescript/rename_project");
        let before = snapshot(temp.path());

        let output = run_batch(temp.path(), batch, extra);

        assert!(!output.status.success());
        let message = common::stderr_text(&output);
        assert!(
            message.contains("Rename 2 of 2 (Counter -> Count in src/lib/util.ts)"),
            "{message}"
        );
        assert_eq!(snapshot(temp.path()), before);
    }
}

#[test]
fn a_batch_that_cannot_work_is_refused_before_the_engine_starts() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    for (batch, expected) in [
        (
            r#"[{"file":"src/a.ts","symbol":"local","new_name":"class"}]"#,
            "reserved word",
        ),
        (
            r#"[{"file":"src/a.ts","symbol":"local","new_name":"x"},{"file":"src/missing.ts","symbol":"a","new_name":"b"}]"#,
            "Rename 2 of 2",
        ),
        (
            r#"[{"file":"src/a.ts","symbol":"local","new_name":"x"},{"file":"src/b.ts","symbol":"dyn","new_name":"2fast"}]"#,
            "not a valid identifier",
        ),
    ] {
        let output = run_batch(project, batch, &[]);
        assert!(!output.status.success());
        let message = common::stderr_text(&output);
        assert!(message.contains(expected), "{message}");
        assert_eq!(snapshot(project), before);
    }
}
