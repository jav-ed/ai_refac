use super::*;

/// The chains of the first macro call in `call`, as the words they hold.
fn chain_words(call: &str) -> Vec<Vec<String>> {
    let code = format!("fn t() {{ {call}; }}\n");
    let parse = SourceFile::parse(&code, Edition::CURRENT);
    let trees = outermost_token_trees(&parse.tree());
    let pieces = pieces_of(&trees[0]);
    chains(&pieces)
        .into_iter()
        .map(|chain| {
            pieces[chain.start..chain.start + chain.len]
                .iter()
                .filter(|piece| piece.text != "::")
                .map(|piece| piece.text.clone())
                .collect()
        })
        .collect()
}

#[test]
fn a_chain_is_a_run_of_words_joined_by_double_colons() {
    assert_eq!(
        chain_words("assert_eq!(a::b::c(), x::y)"),
        vec![vec!["a", "b", "c"], vec!["x", "y"]]
    );
}

#[test]
fn a_lone_word_and_a_struct_field_are_not_chains() {
    assert!(chain_words("m!(value, Item { a: 1 })").is_empty());
}

#[test]
fn super_self_and_crate_start_chains() {
    assert_eq!(
        chain_words("m!(super::a::f(), self::g(), crate::h())"),
        vec![
            vec!["super", "a", "f"],
            vec!["self", "g"],
            vec!["crate", "h"]
        ]
    );
}

#[test]
fn a_chain_after_a_leading_colon_pair_is_not_one() {
    // `<T>::name::x` and `::name::x` begin somewhere this reader cannot see.
    assert!(chain_words("m!(<T>::name::x(), ::name::y())").is_empty());
}

#[test]
fn a_dollar_crate_chain_starts_at_crate() {
    assert_eq!(
        chain_words("m!($crate::engine::f())"),
        vec![vec!["crate", "engine", "f"]]
    );
}
