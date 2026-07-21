mod common;

use std::fs;

#[test]
fn file_move_updates_external_caller_in_project_above_two_thousand_files() {
    let temp = tempfile::tempdir().expect("create temp project");
    let project = temp.path();
    fs::create_dir_all(project.join("src/filler")).expect("create source directories");
    fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext"},"include":["src/**/*.ts"]}"#,
    )
    .expect("write tsconfig");
    fs::write(
        project.join("src/Module.ts"),
        "export const Module_Value = 1;\n",
    )
    .expect("write moved source");
    fs::write(
        project.join("src/Caller.ts"),
        "import { Module_Value } from \"./Module\";\nconsole.log(Module_Value);\n",
    )
    .expect("write external caller");

    // The previous optimization loaded only the moved file above 2,000
    // configured files, returned success, and left this caller stale.
    for index in 0..1_999 {
        fs::write(
            project.join(format!("src/filler/File_{index}.ts")),
            format!("export const value_{index} = {index};\n"),
        )
        .expect("write filler source");
    }

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "src/Module.ts",
        "--target-path",
        "src/core/Module.ts",
    ]);
    common::assert_move_succeeded(&output);

    let caller = common::read_file(project, "src/Caller.ts");
    assert!(
        caller.contains("./core/Module"),
        "external caller remained stale in the large project:\n{caller}"
    );
}
