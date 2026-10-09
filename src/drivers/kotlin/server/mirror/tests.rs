use super::layout::{is_multiplatform, is_multiplatform_set, kotlin_version_in, source_roots};
use super::*;
use serde_json::json;

/// A Gradle root with the given files, relative paths and their text.
pub(super) fn project(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in files {
        let at = dir.path().join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    dir
}

const CATALOG: (&str, &str) = (
    "gradle/libs.versions.toml",
    "[versions]\nkotlin = \"2.1.20\"\n",
);

pub(super) fn uri(path: &Path) -> String {
    Url::from_file_path(path).unwrap().to_string()
}

#[test]
fn source_roots_skip_build_output_and_hidden_folders() {
    let dir = project(&[
        ("shared/src/commonMain/kotlin/a/A.kt", ""),
        ("shared/src/jvmMain/kotlin/a/B.kt", ""),
        ("app/src/main/java/a/C.java", ""),
        ("app/build/generated/src/main/kotlin/D.kt", ""),
        (".git/src/main/kotlin/E.kt", ""),
        ("docs/kotlin/F.kt", ""),
    ]);
    let roots = source_roots(dir.path()).unwrap();
    assert_eq!(
        roots,
        vec![
            PathBuf::from("app/src/main/java"),
            PathBuf::from("shared/src/commonMain/kotlin"),
            PathBuf::from("shared/src/jvmMain/kotlin"),
        ]
    );
}

#[test]
fn multiplatform_source_sets_are_told_from_android_and_jvm_ones() {
    for name in [
        "commonMain",
        "commonTest",
        "jvmMain",
        "androidMain",
        "iosMain",
    ] {
        assert!(is_multiplatform_set(name), "{name}");
    }
    for name in ["main", "test", "androidTest", "debug", "freeRelease"] {
        assert!(!is_multiplatform_set(name), "{name}");
    }
    let android = [
        PathBuf::from("app/src/main/kotlin"),
        PathBuf::from("app/src/androidTest/kotlin"),
    ];
    assert!(!is_multiplatform(&android));
    assert!(is_multiplatform(&[PathBuf::from(
        "shared/src/commonMain/kotlin"
    )]));
}

#[test]
fn the_version_comes_from_the_setting_the_catalog_or_the_plugin_line() {
    let none: [PathBuf; 0] = [];
    let empty = project(&[]);
    assert_eq!(
        kotlin_version_in(Some(" 2.0.21 "), empty.path(), &none).unwrap(),
        "2.0.21"
    );
    let error = kotlin_version_in(Some("latest"), empty.path(), &none)
        .unwrap_err()
        .to_string();
    assert!(error.contains(layout::VERSION_ENV), "{error}");

    let catalog = project(&[CATALOG]);
    assert_eq!(
        kotlin_version_in(None, catalog.path(), &none).unwrap(),
        "2.1.20"
    );

    for (line, expected) in [
        (
            "    kotlin(\"multiplatform\") version \"2.2.0-RC2\"",
            "2.2.0-RC2",
        ),
        (
            "    id(\"org.jetbrains.kotlin.multiplatform\") version \"1.9.24\" apply false",
            "1.9.24",
        ),
        (
            "    classpath(\"org.jetbrains.kotlin:kotlin-gradle-plugin:2.0.0\")",
            "2.0.0",
        ),
    ] {
        let plugin = project(&[("build.gradle.kts", &format!("plugins {{\n{line}\n}}\n"))]);
        assert_eq!(
            kotlin_version_in(None, plugin.path(), &none).unwrap(),
            expected,
            "{line}"
        );
    }
}

#[test]
fn a_version_that_cannot_be_read_is_an_error_naming_the_setting() {
    // A catalog reference is not a version.
    let dir = project(&[(
        "build.gradle.kts",
        "plugins {\n    alias(libs.plugins.kotlinMultiplatform)\n    kotlin(\"multiplatform\") version libs.versions.kotlin\n}\n",
    )]);
    let error = kotlin_version_in(None, dir.path(), &[])
        .unwrap_err()
        .to_string();
    assert!(error.contains(layout::VERSION_ENV), "{error}");
    assert!(error.contains("build.gradle.kts"), "{error}");
}

#[test]
fn a_plain_project_has_no_mirror() {
    let dir = project(&[
        CATALOG,
        ("app/src/main/kotlin/a/A.kt", "package a\n"),
        ("app/src/androidTest/kotlin/a/T.kt", "package a\n"),
    ]);
    assert!(Mirror::for_project(dir.path()).unwrap().is_none());
    assert!(!Mirror::applies_to(dir.path()).unwrap());
}

#[test]
fn the_mirror_holds_every_source_set_under_its_real_path() {
    let dir = project(&[
        CATALOG,
        ("gradlew", "#!/bin/sh\n"),
        (
            "gradle/wrapper/gradle-wrapper.properties",
            "distributionUrl=x\n",
        ),
        ("shared/src/commonMain/kotlin/a/A.kt", "package a\n"),
        ("shared/src/commonMain/composeResources/x.xml", "<x/>"),
        ("shared/src/jvmMain/kotlin/a/B.kt", "package a\n"),
        ("app/src/main/java/a/C.java", "package a;\n"),
    ]);
    let mirror = Mirror::for_project(dir.path()).unwrap().unwrap();
    let root = mirror.root();
    assert!(root.join("shared/src/commonMain/kotlin/a/A.kt").is_file());
    assert!(root.join("shared/src/jvmMain/kotlin/a/B.kt").is_file());
    assert!(root.join("app/src/main/java/a/C.java").is_file());
    assert!(
        !root
            .join("shared/src/commonMain/composeResources/x.xml")
            .exists()
    );
    assert!(
        root.join("gradle/wrapper/gradle-wrapper.properties")
            .is_file()
    );

    let build = std::fs::read_to_string(root.join("build.gradle.kts")).unwrap();
    assert!(
        build.contains("kotlin(\"jvm\") version \"2.1.20\""),
        "{build}"
    );
    assert!(
        build.contains("kotlin.srcDir(\"shared/src/commonMain/kotlin\")"),
        "{build}"
    );
    assert!(
        build.contains("kotlin.srcDir(\"shared/src/jvmMain/kotlin\")"),
        "{build}"
    );
    assert!(
        build.contains("java.srcDir(\"app/src/main/java\")"),
        "{build}"
    );
    assert!(root.join("settings.gradle.kts").is_file());

    let real = dir.path().join("shared/src/jvmMain/kotlin/a/B.kt");
    assert_eq!(mirror.to_real(&mirror.to_mirror(&real)), real);
    assert_eq!(
        mirror.to_mirror(Path::new("/elsewhere/X.kt")),
        PathBuf::from("/elsewhere/X.kt")
    );
}

#[test]
fn uris_are_translated_in_values_and_keys() {
    let dir = project(&[CATALOG, ("shared/src/commonMain/kotlin/a/A.kt", "")]);
    let mirror = Mirror::for_project(dir.path()).unwrap().unwrap();
    let real = uri(&dir.path().join("shared/src/commonMain/kotlin/a/A.kt"));
    let mut message = json!({
        "changes": { real.clone(): [{ "newText": "file:not a path" }] },
        "files": [{ "oldUri": real }],
        "other": "plain text",
    });
    mirror.uris_to_mirror(&mut message);
    let inside = message.to_string();
    assert!(!inside.contains(&*dir.path().to_string_lossy()), "{inside}");
    assert!(inside.contains("file:not a path"), "{inside}");
    assert!(inside.contains("plain text"), "{inside}");
    mirror.uris_to_real(&mut message);
    assert_eq!(message["files"][0]["oldUri"], real);
    assert!(message["changes"].get(&real).is_some(), "{message}");
}

#[test]
fn file_events_keep_the_mirror_like_the_project() {
    let dir = project(&[
        CATALOG,
        ("shared/src/commonMain/kotlin/a/A.kt", "package a\n"),
        ("shared/src/commonMain/kotlin/a/Gone.kt", "package a\n"),
    ]);
    let mirror = Mirror::for_project(dir.path()).unwrap().unwrap();
    let kotlin = dir.path().join("shared/src/commonMain/kotlin");

    // The caller moved a/A.kt to b/A.kt, edited it, deleted Gone.kt and
    // wrote a resource the mirror does not hold.
    std::fs::create_dir_all(kotlin.join("b")).unwrap();
    std::fs::rename(kotlin.join("a/A.kt"), kotlin.join("b/A.kt")).unwrap();
    std::fs::write(kotlin.join("b/A.kt"), "package b\n").unwrap();
    std::fs::remove_file(kotlin.join("a/Gone.kt")).unwrap();
    std::fs::write(kotlin.join("b/strings.xml"), "<x/>").unwrap();
    let forwarded = mirror
        .follow_changes(&json!({ "changes": [
            { "uri": uri(&kotlin.join("a/A.kt")), "type": 3 },
            { "uri": uri(&kotlin.join("b/A.kt")), "type": 1 },
            { "uri": uri(&kotlin.join("a/Gone.kt")), "type": 3 },
            { "uri": uri(&kotlin.join("b/strings.xml")), "type": 1 },
        ] }))
        .unwrap();

    let held = mirror.root().join("shared/src/commonMain/kotlin");
    assert_eq!(
        std::fs::read_to_string(held.join("b/A.kt")).unwrap(),
        "package b\n"
    );
    assert!(!held.join("a/A.kt").exists());
    assert!(!held.join("a/Gone.kt").exists());
    assert!(!held.join("b/strings.xml").exists());
    // The server hears of three events, in its own paths; the resource is not one.
    let events = forwarded["changes"].as_array().unwrap();
    assert_eq!(events.len(), 3, "{forwarded}");
    assert!(events.iter().all(|event| {
        event["uri"]
            .as_str()
            .unwrap()
            .contains(&*mirror.root().to_string_lossy())
    }));
}

#[test]
fn a_created_file_that_is_not_there_is_an_error() {
    let dir = project(&[CATALOG, ("shared/src/commonMain/kotlin/a/A.kt", "")]);
    let mirror = Mirror::for_project(dir.path()).unwrap().unwrap();
    let missing = dir.path().join("shared/src/commonMain/kotlin/a/Missing.kt");
    let error = mirror
        .follow_changes(&json!({ "changes": [{ "uri": uri(&missing), "type": 1 }] }))
        .unwrap_err()
        .to_string();
    assert!(error.contains("Missing.kt"), "{error}");
}

#[test]
fn expect_and_actual_are_blanked_in_the_mirror_and_in_documents_it_is_told_of() {
    let text = "package a\n\nexpect fun f(): String\n";
    let dir = project(&[CATALOG, ("shared/src/commonMain/kotlin/a/A.kt", text)]);
    let mirror = Mirror::for_project(dir.path()).unwrap().unwrap();
    let held = mirror.root().join("shared/src/commonMain/kotlin/a/A.kt");
    let blanked = "package a\n\n       fun f(): String\n";
    assert_eq!(std::fs::read_to_string(&held).unwrap(), blanked);

    let real = dir.path().join("shared/src/commonMain/kotlin/a/A.kt");
    let seen = mirror
        .write_document(&real, "package b\n\nactual fun f(): String = \"\"\n")
        .unwrap();
    assert_eq!(seen, "package b\n\n       fun f(): String = \"\"\n");
    assert_eq!(std::fs::read_to_string(&held).unwrap(), seen);
    // The real file is the caller's and is not written by the mirror.
    assert_eq!(std::fs::read_to_string(&real).unwrap(), text);
}
