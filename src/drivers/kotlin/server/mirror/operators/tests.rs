use super::*;

const HEADER: &str = "package b\n\nimport a.compareTo\nimport a.contains\nimport a.set\n\n";

fn body(text: &str) -> String {
    format!("{HEADER}{text}")
}

#[test]
fn a_comparison_a_membership_test_and_an_indexed_assignment_come_back_as_operators() {
    let before =
        body("fun f(a: Q, b: Q) = a < b\nfun g(a: Q) = 3 in a\nfun h(a: Q) { a[1] = 2 }\n");
    let after = body(
        "fun f(a: Q, b: Q) = a compareTo b\nfun g(a: Q) = 3 contains a\nfun h(a: Q) { a[1] set 2 }\n",
    );

    assert_eq!(undo_operator_calls(&before, &after), before);
}

#[test]
fn a_negated_membership_and_the_longer_comparisons_come_back_too() {
    let before = body("fun f(a: Q) = 3 !in a\nfun g(a: Q, b: Q) = a >= b\n");
    let after = body("fun f(a: Q) = 3 !contains a\nfun g(a: Q, b: Q) = a compareTo b\n");

    assert_eq!(undo_operator_calls(&before, &after), before);
}

#[test]
fn a_call_that_was_written_as_a_call_and_the_imports_are_left_alone() {
    let before = "package a\n\nfun f(a: Q) = a.compareTo(a)\n";
    let after = "package b\n\nimport a.compareTo\n\nfun f(a: Q) = a.compareTo(a)\n";
    assert_eq!(undo_operator_calls(before, after), after);

    // A name that is not an operator function stays what the server made it.
    let before = "package a\n\nfun f() = 1 + 2\n";
    let after = "package a\n\nfun f() = 1 add 2\n";
    assert_eq!(undo_operator_calls(before, after), after);

    // So does a name that replaces a name.
    let before = "package a\n\nfun f(a: Q) = a.plus(a)\n";
    let after = "package a\n\nfun f(a: Q) = a.minus(a)\n";
    assert_eq!(undo_operator_calls(before, after), after);
}

#[test]
fn text_that_is_the_same_is_returned_as_it_is() {
    let text = body("fun f(a: Q) = a < a\n");
    assert_eq!(undo_operator_calls(&text, &text), text);
}
