//! TypeScript symbol rename through the CLI.
//!
//! Fixture: tests/fixtures/typescript/rename_project/
//!
//! util.ts exports `total` (a const), `computeSum`, `Counter` (property `count`,
//! method `increment`), `Options`, `Mode`, `Color`, and a default function.
//! It also declares an unrelated local `total` inside computeSum (line 3), so
//! the name `total` is ambiguous inside that one file.
//!
//! Usages cover: named import, aliased import, namespace import, `export ... from`
//! re-export, `export *`, dynamic import destructuring, shorthand properties, a
//! shadowing parameter, string and comment text, a .js file, and a .tsx file.

use crate::common;

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

mod batch;
mod choosing;
mod encoding;
mod imports;
mod projects;
mod requests;
