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
