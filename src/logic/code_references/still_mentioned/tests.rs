use super::*;
use std::fs;
use tempfile::tempdir;

fn write(root: &Path, file: &str, content: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn scan(root: &Path, sources: &[&str]) -> Vec<String> {
    let sources: Vec<String> = sources.iter().map(|source| source.to_string()).collect();
    Mentions::collect(sources.iter(), root).scan(&HashSet::new())
}

#[test]
fn config_and_data_files_that_name_a_moved_path_are_listed() {
    let dir = tempdir().unwrap();
    write(dir.path(), "src/scripts/menu.ts", "");
    write(
        dir.path(),
        "package.json",
        "{\"scripts\": {\"menu\": \"bun src/scripts/menu.ts\"}}",
    );
    write(
        dir.path(),
        "data/site.yaml",
        "entry: ./src/scripts/menu.ts\n",
    );
    write(
        dir.path(),
        "other.json",
        "{\"a\": \"src/scripts/menu.tsx\"}",
    );
    write(
        dir.path(),
        "src/page.ts",
        "// src/scripts/menu.ts is not config\n",
    );

    let notes = scan(dir.path(), &["src/scripts/menu.ts"]);

    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(
        notes[0].contains("data/site.yaml (src/scripts/menu.ts)"),
        "{notes:?}"
    );
    assert!(
        notes[0].contains("package.json (src/scripts/menu.ts)"),
        "{notes:?}"
    );
    // A longer name that starts alike, and code, are not mentions.
    assert!(!notes[0].contains("other.json"), "{notes:?}");
    assert!(!notes[0].contains("page.ts"), "{notes:?}");
}

#[test]
fn a_folder_matches_the_paths_below_it_and_a_file_does_not() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src/Content/blog")).unwrap();
    write(dir.path(), "src/lib/a.ts", "");
    write(
        dir.path(),
        "tsconfig.json",
        "{\"exclude\": [\"src/Content/blog/drafts\"], \"files\": [\"src/lib/a.ts/extra\"]}",
    );

    assert_eq!(scan(dir.path(), &["src/Content/blog"]).len(), 1);
    assert!(scan(dir.path(), &["src/lib/a.ts"]).is_empty());
}

#[test]
fn a_file_of_public_is_also_found_by_its_url() {
    let dir = tempdir().unwrap();
    write(dir.path(), "Public/brand/logo.svg", "");
    write(
        dir.path(),
        "data/seo.json",
        "{\"image\": \"/brand/logo.svg\", \"other\": \"/brand/logo.svg.bak\"}",
    );

    let notes = scan(dir.path(), &["Public/brand/logo.svg"]);

    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(
        notes[0].contains("data/seo.json (/brand/logo.svg)"),
        "{notes:?}"
    );
}

#[test]
fn files_the_move_edits_lock_files_and_nothing_found_say_nothing() {
    let dir = tempdir().unwrap();
    write(dir.path(), "src/a.ts", "");
    write(dir.path(), "config.json", "{\"a\": \"src/a.ts\"}");
    write(dir.path(), "bun.lock.json", "{\"a\": \"src/a.ts\"}");
    let mentions = Mentions::collect(["src/a.ts".to_string()].iter(), dir.path());

    let edited = HashSet::from([dir.path().join("config.json")]);

    assert!(mentions.scan(&edited).is_empty());
    assert!(scan(dir.path(), &["src/unrelated.ts"]).is_empty());
}

#[test]
fn many_files_are_summarised() {
    let dir = tempdir().unwrap();
    write(dir.path(), "src/a.ts", "");
    for index in 0..13 {
        write(
            dir.path(),
            &format!("c{index:02}.json"),
            "{\"a\": \"src/a.ts\"}",
        );
    }

    let notes = scan(dir.path(), &["src/a.ts"]);

    assert!(notes[0].contains("and 3 more"), "{notes:?}");
}
