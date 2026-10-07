use super::*;
use lsp_types::{Position, Range};
use std::fs;

#[test]
fn reads_every_uri_of_the_leading_directives() {
    let text = "#!/usr/bin/env dart\n// header\n/* block /* nested */ still */\n@Deprecated('x')\nlibrary demo;\n\nimport 'dart:io';\nimport \"package:demo/a.dart\" as a show A;\nimport 'stub.dart' if (dart.library.io) 'io.dart';\nexport '../b.dart' hide B; // trailing\npart 'p.dart';\n\nclass C {}\nimport 'not_a_directive.dart';\n";
    assert_eq!(
        directives(text),
        [
            "dart:io",
            "package:demo/a.dart",
            "stub.dart",
            "io.dart",
            "../b.dart",
            "p.dart"
        ]
    );
    assert_eq!(directives("part of 'lib.dart';\n"), ["lib.dart"]);
    assert!(directives("// import 'a.dart';\nclass X {}\n").is_empty());
    assert!(directives("").is_empty());
}

fn edit(line: u32, from: u32, to: u32, text: &str) -> TextEdit {
    TextEdit {
        range: Range {
            start: Position::new(line, from),
            end: Position::new(line, to),
        },
        new_text: text.to_string(),
    }
}

struct Project {
    dir: tempfile::TempDir,
}

impl Project {
    fn new(files: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        for (path, text) in files {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        Self { dir }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    fn files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = walkdir::WalkDir::new(self.dir.path())
            .into_iter()
            .map(|entry| entry.unwrap().into_path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "dart"))
            .collect();
        files.sort();
        files
    }
}

fn project() -> Project {
    Project::new(&[
        ("pubspec.yaml", "name: demo\n"),
        ("lib/models/m.dart", "class M {}\n"),
        (
            "lib/api.dart",
            "import 'models/m.dart';\nimport 'package:demo/models/m.dart';\n",
        ),
    ])
}

#[test]
fn a_plan_that_rewrites_every_import_is_accepted() {
    let project = project();
    let moves = [(
        project.path("lib/models/m.dart"),
        project.path("lib/domain/m.dart"),
    )];
    let edits = HashMap::from([(
        project.path("lib/api.dart"),
        vec![
            edit(0, 8, 21, "domain/m.dart"),
            edit(1, 21, 34, "domain/m.dart"),
        ],
    )]);
    let dangling = dangling_after(project.dir.path(), &project.files(), &moves, &edits).unwrap();
    assert!(dangling.is_empty(), "{dangling:?}");
}

#[test]
fn an_empty_plan_for_a_referenced_file_is_reported() {
    let project = project();
    let moves = [(
        project.path("lib/models/m.dart"),
        project.path("lib/domain/m.dart"),
    )];
    let dangling = dangling_after(
        project.dir.path(),
        &project.files(),
        &moves,
        &HashMap::new(),
    )
    .unwrap();
    assert_eq!(dangling.len(), 2, "{dangling:?}");
    assert!(dangling[0].contains("lib/api.dart"), "{dangling:?}");
    assert!(dangling[0].contains("models/m.dart"), "{dangling:?}");
}

#[test]
fn a_moved_file_keeps_working_when_its_own_relative_import_is_rewritten() {
    let project = Project::new(&[
        ("pubspec.yaml", "name: demo\n"),
        ("lib/util.dart", "int one() => 1;\n"),
        ("lib/a/user.dart", "import '../util.dart';\n"),
    ]);
    let moves = [(
        project.path("lib/a/user.dart"),
        project.path("lib/b/c/user.dart"),
    )];
    let none = HashMap::new();
    let dangling = dangling_after(project.dir.path(), &project.files(), &moves, &none).unwrap();
    assert_eq!(dangling.len(), 1, "{dangling:?}");
    assert!(dangling[0].contains("lib/b/c/user.dart"), "{dangling:?}");

    let fixed = HashMap::from([(
        project.path("lib/a/user.dart"),
        vec![edit(0, 8, 20, "../../util.dart")],
    )]);
    let dangling = dangling_after(project.dir.path(), &project.files(), &moves, &fixed).unwrap();
    assert!(dangling.is_empty(), "{dangling:?}");
}

#[test]
fn imports_that_were_already_broken_are_not_blamed_on_the_move() {
    let project = Project::new(&[
        ("pubspec.yaml", "name: demo\n"),
        ("lib/a.dart", "class A {}\n"),
        ("lib/old.dart", "import 'gone.dart';\nimport 'a.dart';\n"),
    ]);
    let moves = [(project.path("lib/a.dart"), project.path("lib/z/a.dart"))];
    let edits = HashMap::from([(
        project.path("lib/old.dart"),
        vec![edit(1, 8, 14, "z/a.dart")],
    )]);
    let dangling = dangling_after(project.dir.path(), &project.files(), &moves, &edits).unwrap();
    assert!(dangling.is_empty(), "{dangling:?}");
}

#[test]
fn sdk_other_packages_and_urls_are_not_this_projects_files() {
    let project = Project::new(&[
        ("pubspec.yaml", "name: demo\n"),
        (
            "lib/a.dart",
            "import 'dart:io';\nimport 'package:other/x.dart';\nimport 'https://e.com/x.dart';\n",
        ),
    ]);
    let moves = [(project.path("lib/a.dart"), project.path("lib/b.dart"))];
    let none = HashMap::new();
    let dangling = dangling_after(project.dir.path(), &project.files(), &moves, &none).unwrap();
    assert!(dangling.is_empty(), "{dangling:?}");
}

#[test]
fn the_package_name_comes_from_the_pubspec() {
    let project = Project::new(&[("pubspec.yaml", "description: x\nname: \"demo_app\"\n")]);
    assert_eq!(
        package_name(project.dir.path()).unwrap().as_deref(),
        Some("demo_app")
    );
    let bare = Project::new(&[("lib/a.dart", "")]);
    assert_eq!(package_name(bare.dir.path()).unwrap(), None);
}
