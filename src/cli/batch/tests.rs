use super::*;

fn parse(text: &str) -> Result<(Vec<String>, Vec<String>)> {
    parse_moves(text, Path::new("moves.json"))
}

#[test]
fn the_n_th_source_goes_with_the_n_th_target() {
    let (sources, targets) = parse(
        r#"[{"source_path": "a.kt", "target_path": "p/a.kt"},
            {"source_path": "b.kt", "target_path": "p/b.kt"}]"#,
    )
    .unwrap();

    assert_eq!(sources, ["a.kt", "b.kt"]);
    assert_eq!(targets, ["p/a.kt", "p/b.kt"]);
}

#[test]
fn an_entry_with_a_missing_or_an_unknown_key_is_refused_with_the_expected_shape() {
    for text in [
        r#"[{"source_path": "a.kt"}]"#,
        r#"[{"source_path": "a.kt", "target_path": "b.kt", "line": 3}]"#,
        r#"[{"from": "a.kt", "to": "b.kt"}]"#,
        r#"{"source_path": "a.kt", "target_path": "b.kt"}"#,
        "not json",
    ] {
        let error = format!("{:#}", parse(text).unwrap_err());
        assert!(
            error.contains("moves.json is not a list of moves") && error.contains("target_path"),
            "{text}: {error}"
        );
    }
}

#[test]
fn an_empty_list_is_an_error_and_not_a_move_of_nothing() {
    let error = format!("{:#}", parse("[]").unwrap_err());

    assert!(error.contains("moves.json lists no moves"), "{error}");
}
