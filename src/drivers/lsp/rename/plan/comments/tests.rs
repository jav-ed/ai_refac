use super::*;

fn edited<'a>(before: &'a str, symbol: &'a str, needles: &[&str]) -> EditedText<'a> {
    let spans: Vec<(usize, usize)> = needles
        .iter()
        .map(|needle| {
            let start = before.find(needle).unwrap();
            (start, start + symbol.len())
        })
        .collect();
    EditedText {
        before,
        new_texts: vec!["new"; spans.len()],
        spans,
        symbol,
    }
}

const GO: &str = "// Area reports the area.\ntype Shape interface{ Area() }\nx := 1 // Area here\n";

#[test]
fn a_symbol_on_a_comment_line_is_exempt_and_code_is_not() {
    let text = edited(GO, "Area", &["Area reports", "Area()"]);
    assert_eq!(on_comment_lines(&text, &["//"]), [true, false]);
}

#[test]
fn a_trailing_comment_is_not_a_comment_line() {
    let text = edited(GO, "Area", &["Area here"]);
    assert_eq!(on_comment_lines(&text, &["//"]), [false]);
}

#[test]
fn an_edit_that_is_more_than_the_symbol_is_never_exempt() {
    let before = "// Area reports the area\n";
    let text = EditedText {
        before,
        spans: vec![(3, 3 + "Area reports".len())],
        new_texts: vec!["Surface reports"],
        symbol: "Area",
    };
    assert_eq!(on_comment_lines(&text, &["//"]), [false]);
}

#[test]
fn indented_comments_and_other_markers_work() {
    let before = "def f():\n    # total of items\n    total = 1\n";
    let text = edited(before, "total", &["total of", "total = 1"]);
    assert_eq!(on_comment_lines(&text, &["#"]), [true, false]);
}
