//! Putting a project directory back as it was, and telling what differs, for
//! the tests that keep one Kotlin server across many tests (see `pool`).

use refac::drivers::kotlin::resync::DiskChanges;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Source files by path relative to the project root, with their bytes.
pub type Files = BTreeMap<String, Vec<u8>>;

/// Build and tool state that is no part of what a test compares (the same
/// list as `kotlin::snapshot`).
const STATE: &[&str] = &[".gradle", ".kotlin", ".idea", "build"];

/// Every directory below `root`, relative, except build and tool state.
pub fn directories(root: &Path) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    walk(root, root, &mut found);
    found
}

fn walk(root: &Path, dir: &Path, found: &mut BTreeSet<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() && !STATE.contains(&name.as_str()) {
            found.insert(path.strip_prefix(root).unwrap().display().to_string());
            walk(root, &path, found);
        }
    }
}

/// Make the files and directories of `root` what `baseline` and `directories`
/// say: files of a test are removed, files it moved or edited come back, and
/// the empty directories a move left or made are gone, so the next test sees
/// the project as a fresh copy of the fixture.
pub fn restore(root: &Path, baseline: &Files, directories_of_baseline: &BTreeSet<String>) {
    let now = super::kotlin::snapshot(root);
    for relative in now
        .keys()
        .filter(|relative| !baseline.contains_key(*relative))
    {
        fs::remove_file(root.join(relative)).unwrap();
    }
    for (relative, bytes) in baseline {
        if now.get(relative) != Some(bytes) {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, bytes).unwrap();
        }
    }
    // Deepest first, so that a directory is empty when its turn comes.
    let mut extra: Vec<String> = directories(root)
        .difference(directories_of_baseline)
        .cloned()
        .collect();
    extra.sort_by_key(|relative| std::cmp::Reverse(relative.len()));
    for relative in extra {
        let path = root.join(&relative);
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        }
    }
}

/// What differs between what the server was told (`old`) and the disk (`new`).
pub fn changes(root: &Path, old: &Files, new: &Files) -> DiskChanges {
    let path = |relative: &String| root.join(relative);
    DiskChanges {
        created: new
            .keys()
            .filter(|key| !old.contains_key(*key))
            .map(path)
            .collect(),
        changed: new
            .iter()
            .filter(|(key, bytes)| old.get(*key).is_some_and(|before| before != *bytes))
            .map(|(key, _)| path(key))
            .collect(),
        deleted: old
            .keys()
            .filter(|key| !new.contains_key(*key))
            .map(path)
            .collect(),
    }
}
