mod common;

use std::fs;

#[test]
fn directory_move_updates_external_tsconfig_alias_import() {
    let temp = tempfile::tempdir().expect("create temp project");
    let project = temp.path();

    fs::create_dir_all(project.join("src/Features/Candidate/Registration"))
        .expect("create feature directory");
    fs::create_dir_all(project.join("src/routes")).expect("create routes directory");
    fs::write(
        project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "~/*": ["src/*"] },
    "moduleResolution": "bundler",
    "module": "esnext"
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("write tsconfig");
    fs::write(
        project.join("src/Features/Candidate/Registration/Page.tsx"),
        "export const Candidate_Registration_Page = () => null;\n",
    )
    .expect("write feature");
    fs::write(
        project.join("src/Features/Candidate/Shell.tsx"),
        "export const Candidate_Shell = () => null;\n",
    )
    .expect("write shell");
    fs::write(
        project.join("src/routes/register.lazy.tsx"),
        "import { Candidate_Registration_Page } from \"~/Features/Candidate/Registration/Page\";\n",
    )
    .expect("write external importer");

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "src/Features/Candidate",
        "--target-path",
        "src/Features/Candidate_Portal",
    ]);
    common::assert_move_succeeded(&output);
    let stdout = common::stdout_text(&output);
    assert!(
        stdout.contains("2 TypeScript/JavaScript source files moved (limit: 30)"),
        "directory result did not report the contained source-file count:\n{stdout}"
    );

    let importer = common::read_file(project, "src/routes/register.lazy.tsx");
    assert!(
        importer.contains("~/Features/Candidate_Portal/Registration/Page"),
        "external alias import was not updated:\n{importer}"
    );
    assert!(
        !importer.contains("~/Features/Candidate/Registration/Page"),
        "stale alias import survived:\n{importer}"
    );
}

#[test]
fn file_move_updates_all_external_tsconfig_alias_imports() {
    let temp = tempfile::tempdir().expect("create temp project");
    let project = temp.path();

    fs::create_dir_all(project.join("src/Features/Candidate_Profile"))
        .expect("create profile directory");
    fs::create_dir_all(project.join("src/Features/Candidate_Portal"))
        .expect("create portal directory");
    fs::create_dir_all(project.join("src/Features/Operator_Portal"))
        .expect("create operator directory");
    fs::write(
        project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "~/*": ["src/*"] },
    "moduleResolution": "bundler",
    "module": "esnext"
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("write tsconfig");
    fs::write(
        project.join("src/Features/Candidate_Profile/Empty_Profile.ts"),
        "export const Empty_Profile = {} as const;\n",
    )
    .expect("write profile source");
    for importer in [
        "src/Features/Candidate_Portal/Candidate.ts",
        "src/Features/Operator_Portal/Candidate.ts",
    ] {
        fs::write(
            project.join(importer),
            "import { Empty_Profile } from \"~/Features/Candidate_Profile/Empty_Profile\";\n",
        )
        .expect("write external importer");
    }
    fs::write(
        project.join("src/Explicit_Extension.ts"),
        "import { Empty_Profile } from \"~/Features/Candidate_Profile/Empty_Profile.ts\";\n",
    )
    .expect("write explicit-extension importer");

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "src/Features/Candidate_Profile/Empty_Profile.ts",
        "--target-path",
        "src/Features/Candidate_Profile/Data/Empty_Profile.ts",
    ]);
    common::assert_move_succeeded(&output);

    for importer in [
        "src/Features/Candidate_Portal/Candidate.ts",
        "src/Features/Operator_Portal/Candidate.ts",
    ] {
        let contents = common::read_file(project, importer);
        assert!(
            contents.contains("~/Features/Candidate_Profile/Data/Empty_Profile"),
            "external alias import was not updated in {importer}:\n{contents}"
        );
        assert!(
            !contents.contains("~/Features/Candidate_Profile/Empty_Profile\""),
            "stale alias import survived in {importer}:\n{contents}"
        );
    }

    let explicit = common::read_file(project, "src/Explicit_Extension.ts");
    assert!(
        explicit.contains("~/Features/Candidate_Profile/Data/Empty_Profile.ts"),
        "explicit-extension alias import was not updated:\n{explicit}"
    );
}
