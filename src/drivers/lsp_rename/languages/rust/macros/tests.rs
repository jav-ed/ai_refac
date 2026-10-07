use super::*;

fn slices<'a>(text: &'a str, places: &[UnrenamedPlace]) -> Vec<&'a str> {
    places
        .iter()
        .map(|place| &text[place.start..place.end])
        .collect()
}

#[test]
fn the_body_of_a_macro_is_found_between_its_delimiters() {
    let text = "fn a() {}\nmacro_rules! describe {\n    ($s:expr) => { format!(\"{}\", $s.area()) };\n}\nfn b() { area(); }\n";
    let places = bodies(text);
    assert_eq!(places.len(), 1);
    let body = slices(text, &places)[0];
    assert!(body.contains("$s.area()"), "{body}");
    assert!(!body.contains("fn b"), "{body}");
}

#[test]
fn strings_comments_and_chars_with_brackets_do_not_end_the_body() {
    let text = "macro_rules! m {\n    () => { \"}\" /* } */ // }\n        '}' ; ok };\n}\nafter";
    let places = bodies(text);
    assert_eq!(places.len(), 1);
    let body = slices(text, &places)[0];
    assert!(body.contains("ok };"), "{body}");
    assert!(!body.contains("after"), "{body}");
}

#[test]
fn lifetimes_are_not_character_literals() {
    let text = "macro_rules! m { ($x:expr) => { fn f<'a>(v: &'a str) -> &'a str { v } } }\nafter";
    let body = &bodies(text);
    assert_eq!(body.len(), 1);
    assert!(!slices(text, body)[0].contains("after"));
}

#[test]
fn several_macros_and_round_or_square_delimiters_are_found() {
    let text =
        "macro_rules! a ( () => () );\nmacro_rules! b [ () => () ];\nmacro_rules! c { () => {} }";
    assert_eq!(bodies(text).len(), 3);
}

#[test]
fn text_that_only_mentions_the_keyword_is_not_a_macro() {
    let text = "// uses macro_rules! to define things {\nfn main() {}";
    assert!(bodies(text).is_empty());
}
