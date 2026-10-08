use super::*;

fn rewrite(code: &str) -> String {
    let source = vec!["engine".to_string(), "matching".to_string()];
    let target = vec!["domain".to_string(), "matching".to_string()];
    let mut edits = macro_path_edits(Path::new("a.rs"), code, &source, &target);
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
    let mut result = code.to_string();
    for edit in edits {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    result
}

#[test]
fn paths_in_macro_arguments_are_rewritten() {
    assert_eq!(
        rewrite("fn t() { assert_eq!(crate::engine::matching::value(), 7); }\n"),
        "fn t() { assert_eq!(crate::domain::matching::value(), 7); }\n"
    );
    assert_eq!(
        rewrite(
            "fn t() { let _ = vec![crate::engine::matching::Item { a: 1 }, crate::engine::matching::Item { a: 2 }]; }\n"
        ),
        "fn t() { let _ = vec![crate::domain::matching::Item { a: 1 }, crate::domain::matching::Item { a: 2 }]; }\n"
    );
}

#[test]
fn nested_macros_and_the_module_itself_are_found_once() {
    assert_eq!(
        rewrite("fn t() { f!(g!(crate::engine::matching)); }\n"),
        "fn t() { f!(g!(crate::domain::matching)); }\n"
    );
}

#[test]
fn other_paths_and_ordinary_code_are_left_alone() {
    let code = "use crate::engine::matching::value;\nfn t() { assert_eq!(crate::engine::matchingx::value(), 7); assert_eq!(other::crate::engine::matching::v(), 1); assert_eq!(crate::engine::other::v(), 2); }\n";
    assert_eq!(rewrite(code), code);
}
