//! The documentation site fixture (`tests/fixtures/markdown/site`) and the
//! checks the Markdown move tests share.

use super::links::{edges, moved_edges};
use super::project::{Project, Tree, text};

pub fn site() -> Project {
    Project::from_fixture("markdown/site")
}

/// `rel` of the tree with each pair applied. A pair that matches nothing is a
/// mistake in the test, not a no-op.
pub fn edited(tree: &Tree, rel: &str, pairs: &[(&str, &str)]) -> String {
    let mut content = text(tree, rel);
    for (from, to) in pairs {
        assert!(content.contains(from), "{rel} has no {from:?}");
        content = content.replace(from, to);
    }
    content
}

pub fn sorted(mut paths: Vec<&str>) -> Vec<String> {
    paths.sort();
    paths.into_iter().map(String::from).collect()
}

/// After the move every Markdown file links to the same files as before.
pub fn assert_links_follow(before: &Tree, after: &Tree, place: &dyn Fn(&str) -> String) {
    assert_eq!(
        edges(after),
        moved_edges(&edges(before), place),
        "the links must lead to the same files after the move"
    );
}
