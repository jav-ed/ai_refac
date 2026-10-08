use super::*;

fn rewrite(code: &str, file_module: &[&str], moved: &[&str]) -> String {
    let file_module: Vec<String> = file_module.iter().map(|s| s.to_string()).collect();
    let moved: Vec<String> = moved.iter().map(|s| s.to_string()).collect();
    let mut edits = super_macro_path_edits(Path::new("a.rs"), code, &file_module, &moved);
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
    let mut result = code.to_string();
    for edit in edits {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    result
}

#[test]
fn a_super_path_that_leaves_the_moved_module_becomes_absolute() {
    assert_eq!(
        rewrite(
            "fn t() { assert_eq!(super::helper(), super::super::top()); }\n",
            &["engine", "matching"],
            &["engine", "matching"],
        ),
        "fn t() { assert_eq!(crate::engine::helper(), crate::top()); }\n"
    );
}

#[test]
fn a_super_path_that_stays_inside_the_moved_module_stays() {
    assert_eq!(
        rewrite(
            "mod tests { fn t() { assert_eq!(super::value(), 7); } }\n",
            &["engine", "matching"],
            &["engine", "matching"],
        ),
        "mod tests { fn t() { assert_eq!(super::value(), 7); } }\n"
    );
}

#[test]
fn the_body_of_a_macro_rules_definition_and_a_visibility_are_left_alone() {
    let code =
        "macro_rules! m { () => { super::helper() }; }\nfn t() { declare!(pub(super) fn f()); }\n";
    assert_eq!(
        rewrite(code, &["engine", "matching"], &["engine", "matching"]),
        code
    );
}

#[test]
fn a_super_that_climbs_above_the_crate_root_is_not_touched() {
    let code = "fn t() { assert_eq!(super::super::super::x(), 1); }\n";
    assert_eq!(
        rewrite(code, &["engine", "matching"], &["engine", "matching"]),
        code
    );
}

#[test]
fn a_chain_that_continues_an_earlier_path_is_not_a_super_path() {
    let code = "fn t() { assert_eq!(other::super::x(), 1); }\n";
    assert_eq!(
        rewrite(code, &["engine", "matching"], &["engine", "matching"]),
        code
    );
}
