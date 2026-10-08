use super::*;
use crate::drivers::kotlin::android::survey::survey;
use crate::drivers::lsp::rename::write::journal::Journal;
use std::path::PathBuf;

fn write(root: &Path, relative: &str, text: &str) -> PathBuf {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

/// An Android module whose MainActivity already sits in `ui` after a move.
struct Project {
    dir: tempfile::TempDir,
    moved: Vec<MovedFile>,
}

fn project() -> Project {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "settings.gradle.kts", "include(\":app\")\n");
    write(
        root,
        "build.gradle.kts",
        "plugins { id(\"com.android.application\") version \"9.4.1\" apply false }\n",
    );
    write(
        root,
        "app/build.gradle.kts",
        "android {\n    namespace = \"com.example.droid\"\n}\n",
    );
    write(
        root,
        "app/src/main/AndroidManifest.xml",
        "<manifest><application><activity android:name=\".MainActivity\"/></application></manifest>\n",
    );
    write(
        root,
        "app/src/main/res/layout/main.xml",
        "<LinearLayout tools:context=\".MainActivity\"/>\n",
    );
    let before = "package com.example.droid\n\nimport android.app.Activity\n\nclass MainActivity : Activity() {\n    val name = getString(R.string.app_name)\n}\n";
    let after = before.replace("package com.example.droid", "package com.example.droid.ui");
    let to = write(
        root,
        "app/src/main/java/com/example/droid/ui/MainActivity.kt",
        &after,
    );
    let moved = vec![MovedFile {
        from: root.join("app/src/main/java/com/example/droid/MainActivity.kt"),
        to,
        before: before.to_string(),
        after,
    }];
    Project { dir, moved }
}

fn renames() -> Vec<(String, String)> {
    vec![(
        "com.example.droid.MainActivity".to_string(),
        "com.example.droid.ui.MainActivity".to_string(),
    )]
}

#[test]
fn xml_names_and_the_implicit_r_follow_a_moved_class() {
    let project = project();
    let root = project.dir.path();
    let files = survey(root).unwrap();
    let mut journal = Journal::default();

    let writes = plan(&files, &project.moved, &renames()).unwrap();
    // Planning alone changes nothing.
    let manifest = std::fs::read_to_string(root.join("app/src/main/AndroidManifest.xml")).unwrap();
    assert!(
        manifest.contains("android:name=\".MainActivity\""),
        "{manifest}"
    );
    journal.write_all(&writes).unwrap();

    let manifest = std::fs::read_to_string(root.join("app/src/main/AndroidManifest.xml")).unwrap();
    assert!(
        manifest.contains("android:name=\".ui.MainActivity\""),
        "{manifest}"
    );
    let layout = std::fs::read_to_string(root.join("app/src/main/res/layout/main.xml")).unwrap();
    assert!(
        layout.contains("tools:context=\".ui.MainActivity\""),
        "{layout}"
    );
    let moved = std::fs::read_to_string(&project.moved[0].to).unwrap();
    assert!(moved.contains("import com.example.droid.R\n"), "{moved}");
    assert_eq!(writes.len(), 3);

    // Everything the layer wrote can be undone.
    journal.rollback().unwrap();
    let manifest = std::fs::read_to_string(root.join("app/src/main/AndroidManifest.xml")).unwrap();
    assert!(
        manifest.contains("android:name=\".MainActivity\""),
        "{manifest}"
    );
}

#[test]
fn a_project_without_android_modules_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "build.gradle.kts",
        "plugins { kotlin(\"jvm\") }\n",
    );
    let files = survey(dir.path()).unwrap();

    let writes = plan(&files, &[], &renames()).unwrap();

    assert!(writes.is_empty());
}

#[test]
fn a_manifest_without_a_namespace_in_its_module_is_an_error() {
    let project = project();
    let root = project.dir.path();
    write(
        root,
        "app/build.gradle.kts",
        "plugins { id(\"com.android.application\") }\n",
    );
    let files = survey(root).unwrap();

    let error = plan(&files, &project.moved, &renames()).unwrap_err();

    assert!(error.to_string().contains("namespace"), "{error}");
}

#[test]
fn a_malformed_layout_stops_the_update_with_its_path() {
    let project = project();
    let root = project.dir.path();
    write(
        root,
        "app/src/main/res/layout/broken.xml",
        "<LinearLayout android:id=oops/>\n",
    );
    let files = survey(root).unwrap();

    let error = plan(&files, &project.moved, &renames()).unwrap_err();

    assert!(format!("{error:#}").contains("broken.xml"), "{error:#}");
}
