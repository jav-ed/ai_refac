use super::compare::{edit_count, moved_to};
use super::files::LIMIT_ENV;
use super::*;
use std::fs;
use tempfile::tempdir;

fn project() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("lib/db")).unwrap();
    fs::create_dir_all(root.join("lib/services")).unwrap();
    fs::write(root.join(".gitignore"), "ignored/\n").unwrap();
    fs::write(root.join("lib/db/database.py"), "class Db:\n    pass\n").unwrap();
    fs::write(
        root.join("lib/services/order.py"),
        "from lib.db.database import Db\n\nprint(Db)\n",
    )
    .unwrap();
    fs::write(
        root.join("main.py"),
        "from lib.db.database import Db\n\nx = 1\n\ny = 2\n\nfrom lib.db.database import Db as Other\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("ignored")).unwrap();
    fs::write(root.join("ignored/big.bin"), vec![0u8; 4096]).unwrap();
    dir
}

/// A stand-in for a backend's real move: it renames the file and rewrites the
/// imports in the two files that mention it.
async fn fake_move(pairs: Vec<(String, String)>, copy: PathBuf) -> Result<Vec<String>> {
    assert!(
        !copy.join("ignored").exists(),
        "ignored files are not copied"
    );
    let (from, to) = &pairs[0];
    fs::create_dir_all(copy.join(to).parent().unwrap())?;
    fs::rename(copy.join(from), copy.join(to))?;
    for file in ["lib/services/order.py", "main.py"] {
        let text = fs::read_to_string(copy.join(file))?;
        fs::write(
            copy.join(file),
            text.replace("lib.db.database", "lib.services.database"),
        )?;
    }
    Ok(vec!["a note".to_string()])
}

fn pairs() -> Vec<(String, String)> {
    vec![(
        "lib/db/database.py".to_string(),
        "lib/services/database.py".to_string(),
    )]
}

#[tokio::test]
async fn the_difference_between_the_copy_and_the_project_is_the_preview() {
    let dir = project();
    let preview = preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        fake_move,
    )
    .await
    .unwrap();

    let root = dir.path().canonicalize().unwrap();
    assert_eq!(
        preview.moves,
        vec![(
            root.join("lib/db/database.py"),
            root.join("lib/services/database.py")
        )]
    );
    // order.py has one import changed; main.py has two, far apart: two hunks.
    assert_eq!(preview.edits[&root.join("lib/services/order.py")], 1);
    assert_eq!(preview.edits[&root.join("main.py")], 2);
    assert_eq!(preview.edits.len(), 2);
    assert_eq!(preview.notes, vec!["a note".to_string()]);
}

#[tokio::test]
async fn the_project_itself_is_not_touched() {
    let dir = project();
    let before = fs::read_to_string(dir.path().join("main.py")).unwrap();
    preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        fake_move,
    )
    .await
    .unwrap();
    assert!(dir.path().join("lib/db/database.py").exists());
    assert!(!dir.path().join("lib/services/database.py").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("main.py")).unwrap(),
        before
    );
}

#[tokio::test]
async fn a_failing_move_is_the_error_of_the_dry_run() {
    let dir = project();
    let error = preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        |_, _| async { anyhow::bail!("the backend refuses this") },
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("the backend refuses this"));
}

#[tokio::test]
async fn a_path_outside_the_project_is_refused() {
    let dir = project();
    let error = preview_on_copy(
        Some(dir.path()),
        &[(
            "lib/db/database.py".to_string(),
            "/elsewhere/database.py".to_string(),
        )],
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        fake_move,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("must lie inside"), "{error}");
}

#[tokio::test]
async fn state_a_tool_writes_into_the_copy_is_not_a_change() {
    let dir = project();
    let preview = preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[".ropeproject"],
            scratch: &[],
            ..CopyPlan::default()
        },
        |pairs, copy| async move {
            fs::create_dir_all(copy.join(".ropeproject"))?;
            fs::write(copy.join(".ropeproject/config.py"), "x")?;
            fake_move(pairs, copy).await
        },
    )
    .await
    .unwrap();
    assert_eq!(preview.notes, vec!["a note".to_string()]);
}

#[tokio::test]
async fn files_the_move_creates_or_deletes_are_named() {
    let dir = project();
    let preview = preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        |_, copy| async move {
            fs::write(copy.join("new.py"), "x")?;
            fs::remove_file(copy.join("main.py"))?;
            Ok(Vec::new())
        },
    )
    .await
    .unwrap();
    let text = preview.notes.join("\n");
    assert!(
        text.contains("would create") && text.contains("new.py"),
        "{text}"
    );
    assert!(
        text.contains("would delete") && text.contains("main.py"),
        "{text}"
    );
}

#[test]
fn a_project_over_the_limit_is_refused_with_the_way_to_raise_it() {
    let dir = project();
    let copy = tempdir().unwrap();
    let error = copy_project(dir.path(), copy.path(), 10, &CopyPlan::default()).unwrap_err();
    let text = error.to_string();
    assert!(text.contains(LIMIT_ENV), "{text}");
}

#[test]
fn changes_are_counted_as_hunks() {
    assert_eq!(edit_count(b"a\n", b"a\n"), 0);
    assert_eq!(edit_count(b"a\nb\n", b"a\nc\n"), 1);
    assert_eq!(edit_count(&[0xff, 0x00], &[0xff, 0x01]), 1);
}

#[test]
fn a_moved_folder_carries_what_is_below_it() {
    let pairs = vec![("src/a".to_string(), "src/b".to_string())];
    assert_eq!(
        moved_to(Path::new("src/a/x/y.py"), &pairs),
        PathBuf::from("src/b/x/y.py")
    );
    assert_eq!(
        moved_to(Path::new("src/c.py"), &pairs),
        PathBuf::from("src/c.py")
    );
}

#[tokio::test]
async fn build_output_the_tool_creates_is_not_reported() {
    let dir = project();
    let preview = preview_on_copy(
        Some(dir.path()),
        &pairs(),
        CopyPlan {
            tool_state: &[],
            scratch: &["build"],
            ..CopyPlan::default()
        },
        |pairs, copy| async move {
            fs::create_dir_all(copy.join("build/out"))?;
            fs::write(copy.join("build/out/x.class"), "x")?;
            fake_move(pairs, copy).await
        },
    )
    .await
    .unwrap();
    assert_eq!(preview.notes, vec!["a note".to_string()]);
}

#[cfg(unix)]
#[tokio::test]
async fn a_path_written_through_a_link_to_the_project_is_inside() {
    let dir = project();
    let links = tempdir().unwrap();
    let link = links.path().join("linked");
    std::os::unix::fs::symlink(dir.path(), &link).unwrap();
    // The project is given as the link and so are the absolute paths in it.
    let pairs = vec![(
        link.join("lib/db/database.py")
            .to_string_lossy()
            .into_owned(),
        link.join("lib/services/database.py")
            .to_string_lossy()
            .into_owned(),
    )];
    let preview = preview_on_copy(
        Some(&link),
        &pairs,
        CopyPlan {
            tool_state: &[],
            scratch: &[],
            ..CopyPlan::default()
        },
        fake_move,
    )
    .await
    .unwrap();
    assert_eq!(preview.moves.len(), 1);
    assert_eq!(preview.edits.len(), 2);
}

#[test]
fn only_the_named_extensions_are_copied() {
    let dir = project();
    fs::write(dir.path().join("data.csv"), "1,2\n").unwrap();
    let copy = tempdir().unwrap();
    let copied = copy_project(
        dir.path(),
        copy.path(),
        u64::MAX,
        &CopyPlan {
            only: &["py"],
            ..CopyPlan::default()
        },
    )
    .unwrap();
    assert!(copied.contains(Path::new("lib/db/database.py")));
    assert!(!copied.contains(Path::new("data.csv")));
    assert!(!copy.path().join("data.csv").exists());
    assert!(copy.path().join("main.py").exists());
}

#[test]
fn a_message_about_the_copy_reads_about_the_project() {
    let dir = project();
    let copy = ProjectRoot::at(Some(dir.path()))
        .unwrap()
        .copy(&CopyPlan::default())
        .unwrap();
    let text = format!("Cannot read {}/lib/a.py", copy.path().display());
    let said = copy.about_the_project(&text);
    assert!(!said.contains("refac-dry-run-"), "{said}");
    assert!(
        said.contains(&dir.path().canonicalize().unwrap().display().to_string()),
        "{said}"
    );
}
