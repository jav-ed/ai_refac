mod common;

use std::fs;
use std::process::Command;

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

#[test]
fn five_file_batch_updates_three_thousand_callers_within_one_gib() {
    let temp = tempfile::tempdir().expect("create temp project");
    let project = temp.path();
    fs::create_dir_all(project.join("src/modules")).unwrap();
    fs::create_dir_all(project.join("src/callers")).unwrap();
    fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler"},"include":["src/**/*.ts"]}"#,
    ).unwrap();
    let mut command = Command::new(common::cli_binary());
    command.args([
        "move",
        "--json",
        "--project-path",
        project.to_str().unwrap(),
    ]);
    for index in 0..5 {
        fs::write(
            project.join(format!("src/modules/m{index}.ts")),
            format!(
                "export const v{index} = {index};\nexport {{v{}}} from './m{}';\n",
                (index + 1) % 5,
                (index + 1) % 5
            ),
        )
        .unwrap();
        command.args([
            "--source-path",
            &format!("src/modules/m{index}.ts"),
            "--target-path",
            &format!("src/new/m{index}.ts"),
        ]);
    }
    for index in 0..3_000 {
        fs::write(
            project.join(format!("src/callers/c{index}.ts")),
            format!(
                "// caller {index}\nexport {{v{}}} from '../modules/m{}';\n",
                index % 5,
                index % 5
            ),
        )
        .unwrap();
    }
    // This exercises the installed helper architecture as well as caller coverage.
    // A regression to a project-wide checker must fail under a practical RAM budget.
    let output = command
        .env("REFAC_TYPESCRIPT_MAX_RSS_MB", "1024")
        .output()
        .unwrap();
    common::assert_move_succeeded(&output);
    for index in 0..3_000 {
        assert_eq!(
            common::read_file(project, &format!("src/callers/c{index}.ts")),
            format!(
                "// caller {index}\nexport {{v{}}} from '../new/m{}';\n",
                index % 5,
                index % 5
            ),
        );
    }
    for index in 0..5 {
        assert!(!project.join(format!("src/modules/m{index}.ts")).exists());
        assert_eq!(
            common::read_file(project, &format!("src/new/m{index}.ts")),
            format!(
                "export const v{index} = {index};\nexport {{v{}}} from './m{}';\n",
                (index + 1) % 5,
                (index + 1) % 5
            ),
        );
    }
}
