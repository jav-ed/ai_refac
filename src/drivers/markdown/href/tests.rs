use super::*;

fn parsed(href: &str) -> Option<(&str, &str)> {
    parse(href).map(|reference| (reference.path, reference.suffix))
}

#[test]
fn a_relative_path_is_split_from_its_query_and_fragment() {
    assert_eq!(parsed("docs/a.md"), Some(("docs/a.md", "")));
    assert_eq!(parsed("docs/a.md#intro"), Some(("docs/a.md", "#intro")));
    assert_eq!(parsed("./img/x.png?raw=1"), Some(("./img/x.png", "?raw=1")));
    assert_eq!(parsed("a.md?x=1#top"), Some(("a.md", "?x=1#top")));
    assert_eq!(parsed("../up.md"), Some(("../up.md", "")));
}

#[test]
fn what_is_not_a_relative_file_path_is_not_parsed() {
    for href in [
        "",
        "#section",
        "?query",
        "/site/rooted.md",
        "https://example.com/a.md",
        "http://example.com",
        "mailto:me@example.com",
        "ftp://host/file",
        "data:image/png;base64,AAAA",
        "{{ page.url }}",
        "${base}/a.md",
        "a\\b.md",
        "a&amp;b.md",
        "a&#38;b.md",
    ] {
        assert_eq!(parsed(href), None, "{href}");
    }
}

#[test]
fn an_entity_in_the_query_does_not_hide_the_path() {
    assert_eq!(parsed("a.png?x=1&amp;y=2"), Some(("a.png", "?x=1&amp;y=2")));
    // A bare ampersand is part of a file name.
    assert_eq!(parsed("Q&A.md"), Some(("Q&A.md", "")));
}

#[test]
fn a_drive_letter_or_a_colon_later_in_the_path_is_not_a_scheme() {
    assert!(!has_uri_scheme("C:/docs/a.md"));
    assert!(!has_uri_scheme("docs/a:b.md"));
    assert!(has_uri_scheme("git+ssh://host/x"));
    assert!(!has_uri_scheme("1ab:c"));
}

#[test]
fn resolve_measures_from_the_directory_of_the_file_and_decodes() {
    assert_eq!(
        resolve(Path::new("/p/docs/g.md"), "../img/My%20Logo.png"),
        Some(PathBuf::from("/p/img/My Logo.png"))
    );
    assert_eq!(
        resolve(Path::new("/p/g.md"), "./a/./b.md"),
        Some(PathBuf::from("/p/a/b.md"))
    );
    // A `%` that starts no escape is a `%`.
    assert_eq!(
        resolve(Path::new("/p/g.md"), "100%.md"),
        Some(PathBuf::from("/p/100%.md"))
    );
}

fn written(old: &str, from_dir: &str, target: &str, wrapped: bool) -> String {
    write(old, Path::new(from_dir), Path::new(target), wrapped).unwrap()
}

#[test]
fn the_dot_slash_prefix_follows_the_authors_habit() {
    assert_eq!(written("a.md", "/p", "/p/docs/a.md", false), "docs/a.md");
    assert_eq!(
        written("./a.md", "/p", "/p/docs/a.md", false),
        "./docs/a.md"
    );
    // A path that climbs never gets one.
    assert_eq!(written("./a.md", "/p/x", "/p/a.md", false), "../a.md");
}

#[test]
fn a_directory_link_keeps_its_trailing_slash() {
    assert_eq!(written("docs/", "/p", "/p/manual", false), "manual/");
    assert_eq!(written("./docs/", "/p", "/p/manual", false), "./manual/");
    assert_eq!(written("docs", "/p", "/p/manual", false), "manual");
}

#[test]
fn a_link_to_the_own_directory_is_a_dot() {
    assert_eq!(written("./", "/p/docs", "/p/docs", false), "./");
    assert_eq!(written(".", "/p/docs", "/p/docs", false), ".");
}

#[test]
fn a_colon_in_the_first_segment_gets_a_dot_slash_so_it_is_no_scheme() {
    assert_eq!(written("a.md", "/p", "/p/c:d/a.md", false), "./c:d/a.md");
}

#[test]
fn characters_that_cannot_stand_in_a_destination_are_escaped() {
    assert_eq!(
        written("a.md", "/p", "/p/My Notes.md", false),
        "My%20Notes.md"
    );
    assert_eq!(written("a.md", "/p", "/p/100%.md", false), "100%25.md");
    assert_eq!(written("a.md", "/p", "/p/a[1].md", false), "a%5B1%5D.md");
    assert_eq!(written("a.md", "/p", "/p/a#b.md", false), "a%23b.md");
}

#[test]
fn angle_brackets_keep_a_space_as_it_is_unless_the_author_escaped() {
    assert_eq!(
        written("a b.md", "/p", "/p/My Notes.md", true),
        "My Notes.md"
    );
    assert_eq!(
        written("a%20b.md", "/p", "/p/My Notes.md", true),
        "My%20Notes.md"
    );
}

#[test]
fn non_ascii_letters_stay_as_they_are() {
    assert_eq!(written("a.md", "/p", "/p/gü.md", false), "gü.md");
}

#[test]
fn matching_parentheses_stay_but_a_lone_one_is_escaped() {
    assert_eq!(written("a.md", "/p", "/p/a_(b).md", false), "a_(b).md");
    assert_eq!(written("a.md", "/p", "/p/a_(b.md", false), "a_%28b.md");
    assert_eq!(written("a.md", "/p", "/p/a)b(.md", false), "a%29b%28.md");
}

#[test]
fn entities_are_found_only_when_they_are_complete() {
    assert!(has_html_entity("a&amp;b"));
    assert!(has_html_entity("&#38;"));
    assert!(!has_html_entity("a&b"));
    assert!(!has_html_entity("a&;b"));
    assert!(!has_html_entity("a & b;"));
}
