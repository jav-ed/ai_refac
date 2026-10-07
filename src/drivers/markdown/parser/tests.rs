use super::parse_markdown_links;

fn hrefs(content: &str) -> Vec<String> {
    parse_markdown_links(content)
        .unwrap()
        .into_iter()
        .map(|target| target.href)
        .collect()
}

#[test]
fn parses_inline_links_and_images() {
    let content = "See [Guide](./guide.md) and ![Diagram](./diagram.png).";

    assert_eq!(hrefs(content), ["./guide.md", "./diagram.png"]);
}

#[test]
fn the_range_covers_exactly_the_destination() {
    let content = "Über [Guide](./gü.md#deep \"Title\") ende";
    let targets = parse_markdown_links(content).unwrap();

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].href, "./gü.md#deep");
    assert_eq!(
        &content[targets[0].href_start..targets[0].href_end],
        "./gü.md#deep"
    );
}

#[test]
fn preserves_anchor_fragments() {
    assert_eq!(
        hrefs("[Deep Dive](../docs/guide.md#details)"),
        ["../docs/guide.md#details"]
    );
}

#[test]
fn parses_reference_definitions_in_file_order() {
    let content = "\
See [Guide][guide ref], [overview][], and [shortcut].

[guide ref]: ./guide.md#deep \"Guide Title\"

[overview]: <../overview.md>

[shortcut]: ./shortcut.md
";

    assert_eq!(
        hrefs(content),
        ["./guide.md#deep", "../overview.md", "./shortcut.md"]
    );
}

#[test]
fn a_definition_may_continue_on_the_next_line() {
    assert_eq!(
        hrefs("[multi]:\n   docs/guide.md\n\nSee [multi]."),
        ["docs/guide.md"]
    );
}

#[test]
fn a_definition_continued_inside_a_block_quote_is_read_past_the_markers() {
    assert_eq!(hrefs("> [a]:\n> dest.md\n\n> See [a]."), ["dest.md"]);
    assert_eq!(hrefs("> > [b]:\n> >   deep.md\n"), ["deep.md"]);
}

#[test]
fn inline_and_definition_destinations_come_in_file_order() {
    let content = "[a](./a.md)\n\n[b]: ./b.md\n\n[c](./c.md)\n";

    assert_eq!(hrefs(content), ["./a.md", "./b.md", "./c.md"]);
}

#[test]
fn skips_fenced_code_blocks() {
    assert!(hrefs("before\n```\n[link](./path.md)\n```\nafter").is_empty());
    assert!(hrefs("~~~\n[link](./path.md)\n~~~\n").is_empty());
    assert!(hrefs("```\n[link](./path.md)\n").is_empty());
}

#[test]
fn skips_indented_code_blocks() {
    let content = "Text.\n\n    [x](./path.md)\n\n- item\n\n      [y](./path.md)\n";

    assert!(hrefs(content).is_empty());
}

#[test]
fn skips_inline_code_spans() {
    assert!(hrefs("See `[link](./path.md)` here.").is_empty());
}

#[test]
fn skips_html_comments_and_raw_html() {
    let content =
        "<!-- [c](./path.md) -->\n\n<a href=\"./path.md\">html</a>\n\n<img src=\"./a.png\">\n";

    assert!(hrefs(content).is_empty());
}

#[test]
fn skips_front_matter() {
    assert!(hrefs("---\nlink: [x](./path.md)\n---\n\nBody\n").is_empty());
}

#[test]
fn a_footnote_definition_is_not_a_link_definition() {
    assert!(hrefs("Text[^1].\n\n[^1]: ./path.md is mentioned\n").is_empty());
}

#[test]
fn autolinks_are_not_paths() {
    assert!(hrefs("<https://example.com/doc.md>").is_empty());
}

#[test]
fn finds_both_an_image_and_the_link_around_it() {
    let content = "[![logo](img/logo.png)](docs/guide.md)";

    assert_eq!(hrefs(content), ["img/logo.png", "docs/guide.md"]);
}

#[test]
fn reads_angle_brackets_and_parentheses_in_destinations() {
    assert_eq!(hrefs("[x](<my file.md>)"), ["my file.md"]);
    assert_eq!(hrefs("[x](a_(b).md)"), ["a_(b).md"]);
    assert_eq!(hrefs("[x](a\\)b.md)"), ["a\\)b.md"]);
}

#[test]
fn link_text_with_escapes_code_and_emphasis_does_not_confuse_the_destination() {
    assert_eq!(hrefs("[a\\]b](./one.md)"), ["./one.md"]);
    assert_eq!(hrefs("[`x](y)`](./two.md)"), ["./two.md"]);
    assert_eq!(
        hrefs("[*em* **strong** &amp; <b>raw</b>](./three.md)"),
        ["./three.md"]
    );
    assert_eq!(hrefs("[multi\nline text](./four.md)"), ["./four.md"]);
}

#[test]
fn an_empty_destination_is_not_a_target() {
    assert!(hrefs("[x]()").is_empty());
    assert!(hrefs("[x](<>)").is_empty());
}

#[test]
fn finds_links_in_table_cells_and_list_items() {
    let content = "| a | b |\n|---|---|\n| [x](./x.md) | text |\n\n- [y](./y.md)\n";

    assert_eq!(hrefs(content), ["./x.md", "./y.md"]);
}
