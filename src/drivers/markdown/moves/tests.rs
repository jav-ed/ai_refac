use super::*;

fn file(from: &str, to: &str) -> Move {
    Move {
        from: from.into(),
        to: to.into(),
        is_dir: false,
    }
}

fn dir(from: &str, to: &str) -> Move {
    Move {
        from: from.into(),
        to: to.into(),
        is_dir: true,
    }
}

#[test]
fn a_moved_file_maps_to_its_target_and_nothing_else_does() {
    let set = MoveSet::new(vec![file("/p/a.md", "/p/docs/a.md")]).unwrap();

    assert_eq!(set.map(Path::new("/p/a.md")), Some("/p/docs/a.md".into()));
    assert_eq!(set.map(Path::new("/p/b.md")), None);
    // Below a file there is nothing to carry along.
    assert_eq!(set.map(Path::new("/p/a.md/x")), None);
}

#[test]
fn a_moved_directory_carries_everything_below_it() {
    let set = MoveSet::new(vec![dir("/p/docs", "/p/manual")]).unwrap();

    assert_eq!(set.map(Path::new("/p/docs")), Some("/p/manual".into()));
    assert_eq!(
        set.map(Path::new("/p/docs/img/logo.png")),
        Some("/p/manual/img/logo.png".into())
    );
    // A sibling whose name starts with the same letters is not inside.
    assert_eq!(set.map(Path::new("/p/docs-old/a.md")), None);
}

#[test]
fn destination_is_the_path_itself_when_nothing_moves_it() {
    let set = MoveSet::new(vec![file("/p/a.md", "/p/b.md")]).unwrap();

    assert_eq!(
        set.destination(Path::new("/p/a.md")),
        PathBuf::from("/p/b.md")
    );
    assert_eq!(
        set.destination(Path::new("/p/c.md")),
        PathBuf::from("/p/c.md")
    );
}

#[test]
fn origin_is_the_reverse_of_map() {
    let set = MoveSet::new(vec![
        dir("/p/docs", "/p/manual"),
        file("/p/a.md", "/p/b.md"),
    ])
    .unwrap();

    assert_eq!(
        set.origin(Path::new("/p/manual/x.md")),
        Some("/p/docs/x.md".into())
    );
    assert_eq!(set.origin(Path::new("/p/b.md")), Some("/p/a.md".into()));
    assert_eq!(set.origin(Path::new("/p/other.md")), None);
}

#[test]
fn a_directory_cannot_move_into_itself() {
    let error = MoveSet::new(vec![dir("/p/docs", "/p/docs/old")])
        .unwrap_err()
        .to_string();

    assert!(error.contains("into itself"), "{error}");
}

#[test]
fn two_moves_may_not_compete_for_a_path() {
    let nested_sources = MoveSet::new(vec![
        dir("/p/docs", "/p/a"),
        file("/p/docs/x.md", "/p/b/x.md"),
    ])
    .unwrap_err()
    .to_string();
    assert!(nested_sources.contains("overlap"), "{nested_sources}");

    let nested_targets = MoveSet::new(vec![
        dir("/p/one", "/p/out"),
        file("/p/two.md", "/p/out/two.md"),
    ])
    .unwrap_err()
    .to_string();
    assert!(nested_targets.contains("overlap"), "{nested_targets}");
}

#[test]
fn siblings_with_a_shared_prefix_do_not_overlap() {
    MoveSet::new(vec![dir("/p/docs", "/p/a"), dir("/p/docs-old", "/p/b")]).unwrap();
}

#[test]
fn normalize_resolves_dots_without_touching_the_disk() {
    assert_eq!(
        normalize(Path::new("/p/a/./b/../c.md")),
        PathBuf::from("/p/a/c.md")
    );
    assert_eq!(normalize(Path::new("/p/../../x")), PathBuf::from("/x"));
}

#[test]
fn common_ancestor_is_the_deepest_shared_directory() {
    let paths = [
        Path::new("/p/docs/a"),
        Path::new("/p/docs/b/c"),
        Path::new("/p/docs"),
    ];

    assert_eq!(common_ancestor(paths), PathBuf::from("/p/docs"));
    assert_eq!(
        common_ancestor([Path::new("/p/a"), Path::new("/q/b")]),
        PathBuf::from("/")
    );
}

#[test]
fn absolute_takes_relative_paths_from_the_root() {
    let root = Path::new("/p");

    assert_eq!(
        absolute(Path::new("docs/../a.md"), root),
        PathBuf::from("/p/a.md")
    );
    assert_eq!(
        absolute(Path::new("/q/a.md"), root),
        PathBuf::from("/q/a.md")
    );
}
