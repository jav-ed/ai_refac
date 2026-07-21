mod common;

use std::fs;

#[test]
fn directory_move_rejects_more_than_thirty_source_files_before_mutation() {
    let temp = tempfile::tempdir().expect("create temp project");
    let project = temp.path();
    let source = project.join("src/Feature");
    fs::create_dir_all(&source).expect("create source directory");
    fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{},"include":["src/**/*"]}"#,
    )
    .expect("write tsconfig");

    for index in 0..31 {
        fs::write(
            source.join(format!("File_{index}.ts")),
            format!("export const value_{index} = {index};\n"),
        )
        .expect("write source file");
    }

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "src/Feature",
        "--target-path",
        "src/Feature_Moved",
    ]);
    let stderr = common::stderr_text(&output);

    assert!(!output.status.success(), "oversized move must fail");
    assert!(
        stderr.contains("contains 31 source files; the maximum is 30"),
        "failure must report the measured count:\n{stderr}"
    );
    assert!(source.exists(), "source directory must remain untouched");
    assert!(
        !project.join("src/Feature_Moved").exists(),
        "target directory must not be created"
    );
}
