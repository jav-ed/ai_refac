use crate::common;

// A rename must not move a single character it was not asked to move. The
// server counts columns in UTF-16 units, so text before the symbol on the same
// line (an emoji is two units, a Japanese character one, an accent one) and
// Windows line endings are where an offset mistake would cut the wrong place or
// corrupt a file. Each test puts such text before the declaration and before
// usages, turns every file into CRLF, renames with the real server, and checks
// that the project still builds, the text is intact, and no line ending
// changed.
//
// Run with the servers installed (`refac doctor` explains how):
//   cargo test --test rename encoding:: -- --ignored --test-threads=1

use common::lsp::{at_line, rename, request, require_server};
use std::fs;
use std::path::Path;
use std::process::Command;

/// A character outside the BMP (two UTF-16 units), CJK, and an accent.
const AWKWARD: &str = "😀日本é";

/// Replace `from` with `to` in `file`, which must contain it.
fn replace(project: &Path, file: &str, from: &str, to: &str) {
    let path = project.join(file);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{file} lacks {from:?}");
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

fn files_with(project: &Path, extensions: &[&str]) -> Vec<std::path::PathBuf> {
    fn walk(dir: &Path, extensions: &[&str], out: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if !name.starts_with('.') && name != "target" {
                    walk(&path, extensions, out);
                }
            } else if path
                .extension()
                .is_some_and(|ext| extensions.contains(&ext.to_string_lossy().as_ref()))
            {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(project, extensions, &mut out);
    out
}

fn to_crlf(project: &Path, extensions: &[&str]) {
    for path in files_with(project, extensions) {
        let text = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
        fs::write(path, text.replace('\n', "\r\n")).unwrap();
    }
}

/// Every line of every source file still ends with CRLF.
fn assert_all_crlf(project: &Path, extensions: &[&str]) {
    for path in files_with(project, extensions) {
        let bytes = fs::read(&path).unwrap();
        let lone = bytes
            .iter()
            .enumerate()
            .filter(|(at, byte)| **byte == b'\n' && (*at == 0 || bytes[*at - 1] != b'\r'))
            .count();
        assert_eq!(lone, 0, "{} has {lone} lines without CR", path.display());
    }
}

/// The awkward text is still there, character for character.
fn assert_awkward_kept(project: &Path, extensions: &[&str], expected: usize) {
    let found: usize = files_with(project, extensions)
        .iter()
        .map(|path| fs::read_to_string(path).unwrap().matches(AWKWARD).count())
        .sum();
    assert_eq!(found, expected, "the text {AWKWARD} was damaged");
}

fn run(project: &Path, program: &str, args: &[&str]) -> (bool, String) {
    let output = Command::new(program)
        .args(args)
        .current_dir(project)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {program}: {error}"));
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn go_survives_wide_characters_and_crlf() {
    require_server("go");
    let project = common::setup_fixture("go/rename_module");
    let root = project.path();
    replace(
        root,
        "shape/shape.go",
        "func (r Rect) Area() float64",
        &format!("/* {AWKWARD} */ func (r Rect) Area() float64"),
    );
    replace(
        root,
        "shape/shape.go",
        "total := s.Area()",
        &format!("_ = \"{AWKWARD}\"; total := s.Area()"),
    );
    replace(
        root,
        "cmd/app/main.go",
        "fmt.Println(r.Area(), shape.Count)",
        &format!("fmt.Println(\"{AWKWARD}\", r.Area(), shape.Count)"),
    );
    to_crlf(root, &["go"]);

    let report = rename(at_line(
        request(root, "shape/shape.go", "Area", "Surface"),
        8,
    ))
    .await;

    assert_eq!(report.files.len(), 4, "{:?}", report.files);
    assert_all_crlf(root, &["go"]);
    assert_awkward_kept(root, &["go"], 3);
    for args in [["build", "./..."], ["vet", "./..."]] {
        let (ok, output) = run(root, "go", &args);
        assert!(ok, "go {} failed:\n{output}", args[0]);
    }
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn rust_survives_wide_characters_and_crlf() {
    let project = common::setup_fixture("rust/rename_crate");
    let root = project.path();
    let pin = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml")).unwrap();
    fs::write(root.join("rust-toolchain.toml"), pin).unwrap();
    require_server("rust");
    replace(
        root,
        "src/shapes.rs",
        "    fn area(&self) -> f64;",
        &format!("    /* {AWKWARD} */ fn area(&self) -> f64;"),
    );
    replace(
        root,
        "src/shapes.rs",
        "total += shape.area();",
        &format!("let _ = \"{AWKWARD}\"; total += shape.area();"),
    );
    replace(
        root,
        "src/main.rs",
        "println!(\"{}\", rect.area());",
        &format!("println!(\"{{}} {AWKWARD}\", rect.area());"),
    );
    to_crlf(root, &["rs"]);

    rename(at_line(
        request(root, "src/shapes.rs", "area", "surface"),
        4,
    ))
    .await;

    // The macro body is the documented exception; the test edits it by hand.
    replace(root, "src/report.rs", "$shape.area()", "$shape.surface()");
    assert_all_crlf(root, &["rs"]);
    assert_awkward_kept(root, &["rs"], 3);
    let (ok, output) = run(root, "cargo", &["check", "--all-targets", "--offline"]);
    assert!(ok, "cargo check failed:\n{output}");
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn python_survives_wide_characters_and_crlf() {
    require_server("python");
    let project = common::setup_fixture("python/rename_project");
    let root = project.path();
    replace(
        root,
        "shop/report.py",
        "f\"{type(shape).__name__} with area {shape.area():.2f}\"",
        &format!("f\"{AWKWARD} {{type(shape).__name__}} with area {{shape.area():.2f}}\""),
    );
    replace(
        root,
        "shop/shapes.py",
        "total += shape.area()",
        &format!("\"{AWKWARD}\"; total += shape.area()"),
    );
    to_crlf(root, &["py"]);
    let (ok, before) = run(root, "python3", &["app.py"]);
    assert!(ok, "the project does not run before the rename:\n{before}");

    rename(at_line(
        request(root, "shop/shapes.py", "area", "surface"),
        12,
    ))
    .await;

    assert_all_crlf(root, &["py"]);
    assert_awkward_kept(root, &["py"], 2);
    let (ok, after) = run(root, "python3", &["app.py"]);
    assert!(ok, "the project does not run after the rename:\n{after}");
    // Only the printed class name could differ; the numbers must not.
    assert_eq!(before, after);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn dart_survives_wide_characters_and_crlf() {
    require_server("dart");
    let project = common::setup_fixture("dart/rename_package");
    let root = project.path();
    replace(
        root,
        "lib/shapes.dart",
        "  double area();",
        &format!("  /* {AWKWARD} */ double area();"),
    );
    replace(
        root,
        "lib/shapes.dart",
        "total += shape.area();",
        &format!("final text = '{AWKWARD}'; total += shape.area() + text.length * 0;"),
    );
    replace(
        root,
        "lib/report.dart",
        "'${shape.runtimeType} with area ${shape.area().toStringAsFixed(2)}'",
        &format!(
            "'{AWKWARD} ${{shape.runtimeType}} with area ${{shape.area().toStringAsFixed(2)}}'"
        ),
    );
    to_crlf(root, &["dart"]);
    let (ok, output) = run(root, "dart", &["pub", "get", "--offline"]);
    assert!(ok, "dart pub get failed:\n{output}");

    rename(at_line(
        request(root, "lib/shapes.dart", "area", "surface"),
        7,
    ))
    .await;

    assert_all_crlf(root, &["dart"]);
    assert_awkward_kept(root, &["dart"], 3);
    let (ok, output) = run(root, "dart", &["analyze", "--fatal-infos"]);
    assert!(ok, "dart analyze failed:\n{output}");
}
