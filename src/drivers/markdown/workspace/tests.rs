use super::*;
use std::fs;
use tempfile::tempdir;

fn touch(root: &Path, rel: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "# x\n").unwrap();
}

fn found(root: &Path) -> Vec<String> {
    find(root)
        .unwrap()
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn markdown_extensions_are_recognised_in_any_case() {
    for name in ["a.md", "a.MD", "a.markdown", "a.mdx", "a.Markdown"] {
        assert!(is_markdown(Path::new(name)), "{name}");
    }
    for name in ["a.txt", "md", "a.md.bak", "a.mdown", "README"] {
        assert!(!is_markdown(Path::new(name)), "{name}");
    }
}

#[test]
fn every_markdown_file_below_the_root_is_found_in_path_order() {
    let dir = tempdir().unwrap();
    for rel in [
        "b.md",
        "a/z.markdown",
        "a/y.mdx",
        "a/b/c.md",
        "x.txt",
        "a/img.png",
    ] {
        touch(dir.path(), rel);
    }

    assert_eq!(
        found(dir.path()),
        ["a/b/c.md", "a/y.mdx", "a/z.markdown", "b.md"]
    );
}

#[test]
fn hidden_folders_are_searched_because_documentation_lives_there() {
    let dir = tempdir().unwrap();
    touch(dir.path(), ".github/PULL_REQUEST_TEMPLATE.md");
    touch(dir.path(), ".agents/skills/x/SKILL.md");

    assert_eq!(
        found(dir.path()),
        [
            ".agents/skills/x/SKILL.md",
            ".github/PULL_REQUEST_TEMPLATE.md"
        ]
    );
}

#[test]
fn git_and_node_modules_are_never_searched() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "README.md");
    touch(dir.path(), ".git/notes.md");
    touch(dir.path(), "node_modules/pkg/README.md");
    touch(dir.path(), "web/node_modules/pkg/README.md");

    assert_eq!(found(dir.path()), ["README.md"]);
}

#[test]
fn a_gitignore_inside_the_root_is_honoured_even_without_a_git_repository() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "README.md");
    touch(dir.path(), "build/out.md");
    touch(dir.path(), "docs/draft.md");
    fs::write(dir.path().join(".gitignore"), "build/\ndraft.md\n").unwrap();

    assert_eq!(found(dir.path()), ["README.md"]);
}

#[test]
fn a_nested_gitignore_covers_only_its_own_folder() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "a/skip.md");
    touch(dir.path(), "b/skip.md");
    fs::write(dir.path().join("a/.gitignore"), "skip.md\n").unwrap();

    assert_eq!(found(dir.path()), ["b/skip.md"]);
}

#[test]
fn the_ignore_files_above_the_root_are_not_read() {
    // A project inside a folder that ignores everything is still a project.
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), "*\n").unwrap();
    let project = dir.path().join("project");
    touch(&project, "README.md");

    assert_eq!(found(&project), ["README.md"]);
}

#[tokio::test]
async fn files_that_are_not_utf8_are_reported_not_read() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "good.md");
    fs::write(dir.path().join("bad.md"), [0xff, 0xfe, b'#']).unwrap();

    let workspace = read(vec![dir.path().join("bad.md"), dir.path().join("good.md")])
        .await
        .unwrap();

    assert_eq!(workspace.files.len(), 1);
    assert_eq!(workspace.files[0].path, dir.path().join("good.md"));
    assert_eq!(workspace.unreadable, [dir.path().join("bad.md")]);
}

#[tokio::test]
async fn a_file_that_cannot_be_read_is_an_error() {
    let dir = tempdir().unwrap();

    let error = read(vec![dir.path().join("missing.md")])
        .await
        .err()
        .unwrap()
        .to_string();

    assert!(error.contains("missing.md"), "{error}");
}
