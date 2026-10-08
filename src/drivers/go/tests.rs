use super::requests::*;
use super::*;
use std::fs;

// Environment probe only — run with `cargo test -- --ignored` to verify gopls is installed.
#[tokio::test]
#[ignore]
async fn test_gopls_availability() -> Result<()> {
    let driver = GoDriver::new();
    let avail = driver.check_availability().await?;
    assert!(avail, "gopls not found in PATH");
    Ok(())
}

#[test]
fn test_parse_go_module_path() {
    let go_mod = "module example.com/demo\n\ngo 1.22\n";
    assert_eq!(
        parse_go_module_path(go_mod).as_deref(),
        Some("example.com/demo")
    );
}

#[test]
fn test_find_go_package_name_position() {
    let position =
        find_go_package_name_position("package util\n\nfunc Value() int { return 1 }\n").unwrap();
    assert_eq!(position.line, 0);
    assert_eq!(position.character, 8);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn test_go_move_updates_imports() -> Result<()> {
    let driver = GoDriver::new();
    assert!(driver.check_availability().await?, "gopls not found");

    let temp_dir = tempfile::Builder::new()
        .prefix("refac-go-test-")
        .tempdir_in(std::env::temp_dir())?;
    fs::write(
        temp_dir.path().join("go.mod"),
        "module example.com/demo\n\ngo 1.22\n",
    )?;
    fs::create_dir_all(temp_dir.path().join("util"))?;
    fs::write(
        temp_dir.path().join("util/util.go"),
        "package util\n\nfunc Value() int { return 1 }\n",
    )?;
    fs::write(
        temp_dir.path().join("main.go"),
        "package main\n\nimport (\n    \"fmt\"\n    \"example.com/demo/util\"\n)\n\nfunc main() {\n    fmt.Println(util.Value())\n}\n",
    )?;

    driver
        .move_files(
            vec![("util/util.go".to_string(), "util2/util.go".to_string())],
            Some(temp_dir.path()),
        )
        .await?;

    let main = fs::read_to_string(temp_dir.path().join("main.go"))?;
    let util = fs::read_to_string(temp_dir.path().join("util2/util.go"))?;
    assert!(main.contains("\"example.com/demo/util2\""));
    assert!(main.contains("util2.Value()"));
    assert!(util.contains("package util2"));
    assert!(temp_dir.path().join("util2/util.go").exists());

    Ok(())
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn test_go_move_updates_imports_when_filename_changes() -> Result<()> {
    let driver = GoDriver::new();
    assert!(driver.check_availability().await?, "gopls not found");

    let temp_dir = tempfile::Builder::new()
        .prefix("refac-go-rename-test-")
        .tempdir_in(std::env::temp_dir())?;
    fs::write(
        temp_dir.path().join("go.mod"),
        "module example.com/demo\n\ngo 1.22\n",
    )?;
    fs::create_dir_all(temp_dir.path().join("util"))?;
    fs::write(
        temp_dir.path().join("util/util.go"),
        "package util\n\nfunc Value() int { return 1 }\n",
    )?;
    fs::write(
        temp_dir.path().join("main.go"),
        "package main\n\nimport (\n    \"fmt\"\n    \"example.com/demo/util\"\n)\n\nfunc main() {\n    fmt.Println(util.Value())\n}\n",
    )?;

    driver
        .move_files(
            vec![("util/util.go".to_string(), "util2/renamed.go".to_string())],
            Some(temp_dir.path()),
        )
        .await?;

    let main = fs::read_to_string(temp_dir.path().join("main.go"))?;
    let renamed = fs::read_to_string(temp_dir.path().join("util2/renamed.go"))?;
    assert!(main.contains("\"example.com/demo/util2\""));
    assert!(main.contains("util2.Value()"));
    assert!(renamed.contains("package util2"));
    assert!(temp_dir.path().join("util2/renamed.go").exists());
    assert!(!temp_dir.path().join("util/util.go").exists());

    let go_build = std::process::Command::new("go")
        .args(["build", "./..."])
        .current_dir(temp_dir.path())
        .output()?;
    assert!(
        go_build.status.success(),
        "go build failed: {}",
        String::from_utf8_lossy(&go_build.stderr)
    );

    Ok(())
}
