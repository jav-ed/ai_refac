use super::*;

/// What a move of `moves` does to `content`, written for `/p/docs/g.md` and
/// staying there unless `file_to` says otherwise.
fn rewritten(content: &str, file_to: Option<&str>, moves: &[(&str, &str)]) -> Rewritten {
    let original = Path::new("/p/docs/g.md");
    let final_path = Path::new(file_to.unwrap_or("/p/docs/g.md"));
    let destination = |path: &Path| {
        moves
            .iter()
            .find(|(from, _)| Path::new(from) == path)
            .map_or_else(|| path.to_path_buf(), |(_, to)| PathBuf::from(to))
    };
    rewrite(content, original, final_path, &destination).unwrap()
}

#[test]
fn a_link_follows_the_file_it_points_at() {
    let result = rewritten(
        "See [a](a.md#top) and [b](b.md).",
        None,
        &[("/p/docs/a.md", "/p/docs/sub/a.md")],
    );

    assert_eq!(result.content, "See [a](sub/a.md#top) and [b](b.md).");
    assert_eq!(result.links_updated, 1);
}

#[test]
fn nothing_changes_when_nothing_a_link_points_at_moves() {
    let content = "[a](a.md) <img src=\"x.png\"> [x]: ./y.md\n";
    let result = rewritten(content, None, &[("/p/other.md", "/p/else.md")]);

    assert_eq!(result.content, content);
    assert_eq!(result.links_updated, 0);
}

#[test]
fn a_moved_file_keeps_its_links_pointing_at_the_same_files() {
    let result = rewritten(
        "[a](a.md) [up](../README.md) [img](img/x.png)",
        Some("/p/manual/deep/g.md"),
        &[],
    );

    assert_eq!(
        result.content,
        "[a](../../docs/a.md) [up](../../README.md) [img](../../docs/img/x.png)"
    );
    assert_eq!(result.links_updated, 3);
}

#[test]
fn a_link_between_two_files_that_move_together_is_left_as_it_was() {
    // `docs/` and the target move to `manual/`; the link was spelled oddly and
    // still says the same thing, so it is not touched.
    let result = rewritten(
        "[a](x/../a.md) [b](./sub//b.md)",
        Some("/p/manual/g.md"),
        &[
            ("/p/docs/a.md", "/p/manual/a.md"),
            ("/p/docs/sub/b.md", "/p/manual/sub/b.md"),
        ],
    );

    assert_eq!(result.content, "[a](x/../a.md) [b](./sub//b.md)");
    assert_eq!(result.links_updated, 0);
}

#[test]
fn urls_anchors_and_site_paths_are_never_touched() {
    let content = "[w](https://e.com/a.md) [a](#top) [r](/abs/a.md) [m](mailto:x@y.z)";
    let result = rewritten(content, Some("/p/else/g.md"), &[]);

    assert_eq!(result.content, content);
}

#[test]
fn reference_definitions_html_and_images_are_rewritten_too() {
    let content = "![i](a.png)\n\n<img src=\"a.png\" srcset=\"a.png 1x, b.png 2x\">\n\n[ref]: <a.png> \"T\"\n";
    let result = rewritten(
        content,
        None,
        &[
            ("/p/docs/a.png", "/p/img/a.png"),
            ("/p/docs/b.png", "/p/img/b.png"),
        ],
    );

    assert_eq!(
        result.content,
        "![i](../img/a.png)\n\n<img src=\"../img/a.png\" srcset=\"../img/a.png 1x, ../img/b.png 2x\">\n\n[ref]: <../img/a.png> \"T\"\n"
    );
    assert_eq!(result.links_updated, 5);
}

#[test]
fn several_links_with_different_lengths_on_one_line_stay_aligned() {
    let result = rewritten(
        "[a](a.md)[b](b.md)[c](c.md)",
        None,
        &[
            ("/p/docs/a.md", "/p/docs/very/long/new/a.md"),
            ("/p/docs/c.md", "/p/c.md"),
        ],
    );

    assert_eq!(
        result.content,
        "[a](very/long/new/a.md)[b](b.md)[c](../c.md)"
    );
}

#[test]
fn percent_encoding_and_the_query_survive() {
    let result = rewritten(
        "[n](My%20Notes.md?raw=1#top)",
        None,
        &[("/p/docs/My Notes.md", "/p/docs/Sub Dir/My Notes.md")],
    );

    assert_eq!(result.content, "[n](Sub%20Dir/My%20Notes.md?raw=1#top)");
}

#[test]
fn text_around_the_links_is_byte_for_byte_the_same() {
    let content = "Über ünïcode [a](a.md) — “quotes”\r\n\tTabs\r\n";
    let result = rewritten(content, None, &[("/p/docs/a.md", "/p/docs/b/a.md")]);

    assert_eq!(
        result.content,
        "Über ünïcode [a](b/a.md) — “quotes”\r\n\tTabs\r\n"
    );
}
