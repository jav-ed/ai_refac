//! The repair itself, on small projects written to a temporary folder.

use super::*;

fn write(root: &Path, relative: &str, text: &str) -> PathBuf {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

/// A project: the files that stayed, plus one moved file given as its text
/// before and after the move (as the server left it).
struct Project {
    dir: tempfile::TempDir,
    stayed: Vec<PathBuf>,
}

impl Project {
    fn new(stayed: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "build.gradle.kts", "");
        let stayed = stayed
            .iter()
            .map(|(path, text)| write(dir.path(), path, text))
            .collect();
        Self { dir, stayed }
    }

    fn build_files(&self) -> Vec<PathBuf> {
        walkdir::WalkDir::new(self.dir.path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() == "build.gradle.kts")
            .map(walkdir::DirEntry::into_path)
            .collect()
    }

    /// Run the repair for a file moved to `to`. Returns its text afterwards.
    fn repair(&self, from: &str, to: &str, before: &str, after: &str) -> (String, Vec<String>) {
        let to_path = write(self.dir.path(), to, after);
        let mut sources = self.stayed.clone();
        sources.push(to_path.clone());
        let survey = Survey {
            build_files: self.build_files(),
            sources,
            ..Survey::default()
        };
        let moved = vec![MovedFile {
            from: self.dir.path().join(from),
            to: to_path.clone(),
            before: before.to_string(),
            after: after.to_string(),
        }];
        let mut writes = Vec::new();
        let notes = plan(&survey, &moved, &mut writes).unwrap();
        let text = match writes.iter().find(|write| write.path == to_path) {
            Some(write) => String::from_utf8(write.bytes.clone()).unwrap(),
            None => after.to_string(),
        };
        (text, notes)
    }
}

const OLD: &str = "src/commonMain/kotlin/com/x/util";
const NEW: &str = "src/commonMain/kotlin/com/x/util/sub";

/// The failure that was found: `platformName` is an `expect` in one file and an
/// `actual` in another, and the server did not import it.
#[test]
fn an_expect_actual_pair_and_overloads_are_imported_by_name() {
    let project = Project::new(&[
        (
            &format!("{OLD}/Platform.kt"),
            "package com.x.util\n\nexpect fun platformName(): String\nfun twice(a: String) = a\nfun twice(a: String, b: String) = a\nval LIMIT = 3\nopen class Base\n",
        ),
        (
            "src/jvmMain/kotlin/com/x/util/Jvm.kt",
            "package com.x.util\n\nactual fun platformName(): String = \"jvm\"\n",
        ),
    ]);
    let before = "package com.x.util\n\nimport com.x.util.shout\n\nfun go() = platformName() + twice(\"a\") + LIMIT + Base()\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, notes) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    for name in ["platformName", "twice", "LIMIT", "Base"] {
        assert!(
            text.contains(&format!("import com.x.util.{name}\n")),
            "{name} is not imported:\n{text}"
        );
    }
    assert!(text.contains("import com.x.util.shout\n"), "{text}");
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("platformName"), "{notes:?}");
}

#[test]
fn what_is_imported_declared_by_the_file_private_or_not_used_is_left_alone() {
    let project = Project::new(&[(
        &format!("{OLD}/Stay.kt"),
        "package com.x.util\n\nfun used() = 1\nfun unused() = 2\nprivate fun secret() = 3\nclass Local\nfun member() = 4\nval imported = 5\n",
    )]);
    let before = "package com.x.util\n\nimport other.imported\n\nclass Local\n\nfun go(o: Obj) = used() + secret() + o.member() + imported + \"unused()\" // unused()\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, notes) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    assert!(text.contains("import com.x.util.used\n"), "{text}");
    assert!(
        !text.contains("import com.x.util.secret"),
        "a private name is not visible: {text}"
    );
    assert!(!text.contains("import com.x.util.unused"), "{text}");
    assert!(
        !text.contains("import com.x.util.Local"),
        "its own class: {text}"
    );
    assert!(
        !text.contains("import com.x.util.member"),
        "a member call: {text}"
    );
    assert!(
        !text.contains("import com.x.util.imported"),
        "`imported` already means other.imported: {text}"
    );
    assert_eq!(notes.len(), 1);
}

#[test]
fn an_extension_function_is_imported_when_it_is_called_after_a_dot() {
    let project = Project::new(&[(
        &format!("{OLD}/Ext.kt"),
        "package com.x.util\n\nfun String.tag() = this\nfun Int.tag() = this\nfun plain() = 1\n",
    )]);
    let before = "package com.x.util\n\nfun go() = \"a\".tag() + 1.plain()\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, _) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    assert!(text.contains("import com.x.util.tag\n"), "{text}");
    assert!(!text.contains("import com.x.util.plain"), "{text}");
}

#[test]
fn a_file_with_no_imports_gets_them_below_its_package_line() {
    let project = Project::new(&[(
        &format!("{OLD}/Stay.kt"),
        "package com.x.util\n\nfun shout() = 1\n",
    )]);
    let before = "package com.x.util\n\nfun go() = shout()\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, _) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    assert_eq!(
        text,
        "package com.x.util.sub\n\nimport com.x.util.shout\n\nfun go() = shout()\n"
    );
}

#[test]
fn names_from_another_module_a_test_set_or_a_sibling_platform_are_not_imported() {
    let project = Project::new(&[
        (
            "src/test/kotlin/com/x/util/Fixture.kt",
            "package com.x.util\n\nfun fixture() = 1\n",
        ),
        (
            "src/androidMain/kotlin/com/x/util/Droid.kt",
            "package com.x.util\n\nfun droid() = 1\n",
        ),
        (
            "other/src/main/kotlin/com/x/util/Elsewhere.kt",
            "package com.x.util\n\nfun elsewhere() = 1\n",
        ),
    ]);
    write(project.dir.path(), "other/build.gradle.kts", "");
    let before = "package com.x.util\n\nfun go() = fixture() + droid() + elsewhere()\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, notes) = project.repair(
        "src/jvmMain/kotlin/com/x/util/Moving.kt",
        "src/jvmMain/kotlin/com/x/util/sub/Moving.kt",
        before,
        &after,
    );

    assert_eq!(text, after);
    assert!(notes.is_empty(), "{notes:?}");
}

#[test]
fn a_java_class_of_the_old_package_is_imported() {
    let project = Project::new(&[(
        "src/main/java/com/x/util/Legacy.java",
        "package com.x.util;\n\npublic class Legacy {}\n",
    )]);
    let before = "package com.x.util\n\nfun go(l: Legacy) = l\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, _) = project.repair(
        "src/main/kotlin/com/x/util/Moving.kt",
        "src/main/kotlin/com/x/util/sub/Moving.kt",
        before,
        &after,
    );

    assert!(text.contains("import com.x.util.Legacy\n"), "{text}");
}

#[test]
fn a_file_that_keeps_its_package_is_not_touched_and_the_android_writes_are_built_upon() {
    let project = Project::new(&[(
        &format!("{OLD}/Stay.kt"),
        "package com.x.util\n\nfun shout() = 1\n",
    )]);
    let same = "package com.x.util\n\nfun go() = shout()\n";
    let (text, notes) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{OLD}/Renamed.kt"),
        same,
        same,
    );
    assert_eq!(text, same);
    assert!(notes.is_empty());

    // The Android layer wrote an import of R into the moved file first.
    let after = "package com.x.util.sub\n\nimport com.x.R\n\nfun go() = shout() + R.id.a\n";
    let to = write(project.dir.path(), &format!("{NEW}/Moving.kt"), after);
    let survey = Survey {
        build_files: project.build_files(),
        sources: vec![project.stayed[0].clone(), to.clone()],
        ..Survey::default()
    };
    let moved = vec![MovedFile {
        from: project.dir.path().join(format!("{OLD}/Moving.kt")),
        to: to.clone(),
        before: "package com.x.util\n\nfun go() = shout() + R.id.a\n".to_string(),
        after: after.to_string(),
    }];
    let mut writes = vec![FileWrite {
        path: to.clone(),
        bytes: after.as_bytes().to_vec(),
        changes: 1,
    }];

    plan(&survey, &moved, &mut writes).unwrap();

    assert_eq!(writes.len(), 1, "one write for the file");
    let text = String::from_utf8(writes[0].bytes.clone()).unwrap();
    assert!(text.contains("import com.x.R\n") && text.contains("import com.x.util.shout\n"));
    assert_eq!(writes[0].changes, 2);
}
