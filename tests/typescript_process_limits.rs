mod common;

use std::process::Command;

#[test]
fn invalid_memory_limits_fail_before_moving_files() {
    for value in ["0", "garbage", "18446744073709551615"] {
        let temp = common::setup_fixture("typescript/project");
        let output = Command::new(common::cli_binary())
            .args([
                "move",
                "--project-path",
                temp.path().to_str().unwrap(),
                "--source-path",
                "src/utils/date_helpers.ts",
                "--target-path",
                "src/lib/date_helpers.ts",
            ])
            .env("REFAC_TYPESCRIPT_MAX_RSS_MB", value)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(common::stderr_text(&output).contains("must be a positive integer in MiB"));
        assert!(temp.path().join("src/utils/date_helpers.ts").exists());
        assert!(!temp.path().join("src/lib/date_helpers.ts").exists());
    }
}

#[test]
fn memory_failure_reports_the_limit_and_cleanup_through_cli() {
    let temp = common::setup_fixture("typescript/project");
    // Keep the helper alive beyond an RSS sample even with the faster parser.
    for index in 0..3_000 {
        std::fs::write(
            temp.path().join(format!("src/caller_{index}.ts")),
            "export * from './utils/date_helpers';\n",
        )
        .unwrap();
    }
    let output = Command::new(common::cli_binary())
        .args([
            "move",
            "--project-path",
            temp.path().to_str().unwrap(),
            "--source-path",
            "src/utils/date_helpers.ts",
            "--target-path",
            "src/lib/date_helpers.ts",
        ])
        .env("REFAC_TYPESCRIPT_MAX_RSS_MB", "1")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let message = common::stderr_text(&output);
    assert!(message.contains("exceeded its RAM limit"), "{message}");
    assert!(message.contains("terminated and reaped"), "{message}");
}
