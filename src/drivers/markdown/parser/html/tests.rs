use super::*;

/// The values found when `pieces` arrive one after the other, the way the
/// Markdown parser hands over a block of HTML line by line.
fn found(pieces: &[&str]) -> Vec<String> {
    let mut scanner = HtmlScanner::default();
    let mut ranges = Vec::new();
    let mut text = String::new();
    for piece in pieces {
        scanner.scan(piece, text.len(), &mut ranges);
        text.push_str(piece);
    }
    ranges
        .into_iter()
        .map(|range| text[range].to_string())
        .collect()
}

#[test]
fn href_src_and_poster_are_read_quoted_or_bare() {
    assert_eq!(found(&["<a href=\"a.md\">x</a>"]), ["a.md"]);
    assert_eq!(found(&["<img src='a.png'>"]), ["a.png"]);
    assert_eq!(found(&["<img src=a.png alt=x>"]), ["a.png"]);
    assert_eq!(
        found(&["<video poster=\"p.jpg\" src=\"v.mp4\"></video>"]),
        ["p.jpg", "v.mp4"]
    );
    assert_eq!(found(&["<A HREF = \"a.md\">"]), ["a.md"]);
}

#[test]
fn every_srcset_candidate_is_a_path() {
    assert_eq!(
        found(&["<img srcset=\"a.png 1x,  b.png 2x, c.png\">"]),
        ["a.png", "b.png", "c.png"]
    );
}

#[test]
fn other_attributes_are_not_paths() {
    assert_eq!(
        found(&["<div data-src=\"x.png\" class=\"a.md\" title=\"href=b.md\">"]),
        Vec::<String>::new()
    );
}

#[test]
fn an_empty_value_is_not_a_path() {
    assert!(found(&["<a href=\"\">"]).is_empty());
}

#[test]
fn comments_hide_what_they_contain() {
    assert!(found(&["<!-- <a href=\"a.md\"> -->"]).is_empty());
    assert_eq!(found(&["<!-- x -->", "<a href=\"b.md\">"]), ["b.md"]);
}

#[test]
fn a_comment_may_span_pieces() {
    assert!(found(&["<!-- start", "<a href=\"a.md\">", "end -->"]).is_empty());
    assert_eq!(
        found(&["<!-- start", "end -->", "<a href=\"b.md\">"]),
        ["b.md"]
    );
}

#[test]
fn a_tag_may_continue_on_the_next_piece() {
    assert_eq!(found(&["<img", " src=\"a.png\"", " alt=\"x\">"]), ["a.png"]);
    assert_eq!(
        found(&["<img src=", "\"a.png\">"]).len(),
        0,
        "a value cut by the piece boundary is not guessed"
    );
}

#[test]
fn code_and_script_content_is_text() {
    assert!(found(&["<pre>", "<a href=\"a.md\">", "</pre>"]).is_empty());
    assert!(found(&["<code><a href=\"a.md\"></code>"]).is_empty());
    assert!(found(&["<script>", "var u = '<a href=\"a.md\">';", "</script>"]).is_empty());
    assert_eq!(found(&["<pre>x</pre><a href=\"b.md\">"]), ["b.md"]);
}

#[test]
fn a_closing_tag_has_no_paths() {
    assert!(found(&["</a href=\"a.md\">"]).is_empty());
}

#[test]
fn a_lone_angle_bracket_is_text() {
    assert_eq!(found(&["a < b and <a href=\"c.md\">"]), ["c.md"]);
}

#[test]
fn the_ranges_are_offsets_into_the_whole_file() {
    let mut scanner = HtmlScanner::default();
    let mut ranges = Vec::new();
    scanner.scan("<a href=\"x.md\">", 100, &mut ranges);

    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 109..113);
}

#[test]
fn something_that_is_not_html_ends_an_open_tag() {
    let mut scanner = HtmlScanner::default();
    let mut ranges = Vec::new();
    scanner.scan("<img", 0, &mut ranges);
    scanner.interrupted();
    scanner.scan(" src=\"a.png\">", 4, &mut ranges);

    assert!(ranges.is_empty());
}
