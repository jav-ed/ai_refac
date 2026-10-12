// Operators the syntax calls, and a package split over two Gradle modules, against the
// real Kotlin language server. Fixtures: jvm_project/.../ops/ (Qty.kt declares the operators,
// Calc.kt uses them) and multi_module/ (a library and an app that share a package name).
// Run with: REFAC_KOTLIN_SERVER=<install dir> REFAC_KOTLIN_TESTS=all cargo test --test kotlin moves::operators -- --ignored --test-threads=1

use super::{COMPILE, pair, run, text};
use crate::common;

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn operators_keep_their_syntax_and_get_the_imports_of_the_functions_they_call() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(project.path(), &[pair("ops/Calc.kt", "ops/calc/Calc.kt")]).await;

    // The server turned `<`, `in` and `a[1] = 2` into infix calls; a move does
    // not rewrite operators.
    let calc = text(project.path(), "ops/calc/Calc.kt");
    assert!(calc.starts_with("package com.example.ops.calc"), "{calc}");
    for operator in [
        "a < b",
        "3 in a",
        "3 !in a",
        "a[1]",
        "a[1] = 2",
        "val (x, y) = a",
        "for (value in bag)",
        "var n by registry",
    ] {
        assert!(calc.contains(operator), "{operator} is gone: {calc}");
    }
    // The server imports the operators written as symbols; the layer after it
    // imports the ones the syntax calls without a symbol: `get`, `component1`,
    // `component2`, `iterator`, `getValue` and `setValue`.
    for function in [
        "compareTo",
        "contains",
        "set",
        "get",
        "component1",
        "component2",
        "iterator",
        "getValue",
        "setValue",
    ] {
        assert!(
            calc.contains(&format!("import com.example.ops.{function}\n")),
            "import of {function} is missing: {calc}"
        );
    }
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn the_files_left_behind_get_the_imports_of_the_operators_that_moved_away() {
    let project = common::pool::lease("kotlin/jvm_project").await;

    run(project.path(), &[pair("ops/Qty.kt", "ops/model/Qty.kt")]).await;

    // Calc.kt stayed in `ops` and uses the operators of Qty.kt through `a[1]`,
    // a destructuring, a loop and `by`. The server imports `compareTo` and
    // `contains`, which are written as symbols; refac imports the rest.
    let calc = text(project.path(), "ops/Calc.kt");
    for function in [
        "compareTo",
        "contains",
        "get",
        "set",
        "component1",
        "component2",
        "iterator",
        "getValue",
        "setValue",
    ] {
        assert!(
            calc.contains(&format!("import com.example.ops.model.{function}\n")),
            "import of {function} is missing: {calc}"
        );
    }
    for operator in ["a < b", "3 in a", "a[1] = 2", "var n by registry"] {
        assert!(calc.contains(operator), "{operator} is gone: {calc}");
    }
    project.assert_compiles(COMPILE);
}

#[tokio::test]
#[ignore = "needs the Kotlin language server (REFAC_KOTLIN_SERVER) and a JDK"]
async fn a_package_split_over_two_modules_gets_the_imports_of_the_library() {
    let project = common::pool::lease("kotlin/multi_module").await;

    run(
        project.path(),
        &[(
            "app/src/main/kotlin/com/example/shared/Screen.kt".to_string(),
            "app/src/main/kotlin/com/example/screens/Screen.kt".to_string(),
        )],
    )
    .await;

    // Screen.kt is in the app and the rest of `com.example.shared` is in the
    // library the app depends on. The server imports the names (`LibBox`,
    // `libHelper`) and `plus`; refac imports `get`, which `box[1]` calls.
    let screen = common::read_file(
        project.path(),
        "app/src/main/kotlin/com/example/screens/Screen.kt",
    );
    for name in ["LibBox", "libHelper", "plus", "get"] {
        assert!(
            screen.contains(&format!("import com.example.shared.{name}\n")),
            "import of {name} is missing: {screen}"
        );
    }
    project.assert_compiles(COMPILE);
}
