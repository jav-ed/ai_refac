//! Operators and delegates that the syntax calls, and the modules they live in.

use super::*;

/// The server imports `+` and `<` but not `a[i]`, a `for` loop, a destructuring
/// declaration or `by` (found on the real server, 2026-10-12). Only an
/// `operator` extension of the old package is imported for them.
#[test]
fn operators_the_syntax_calls_are_imported_when_the_old_package_declares_them() {
    let project = Project::new(&[(
        &format!("{OLD}/Ops.kt"),
        "package com.x.util\n\nclass Qty\nclass Bag\nclass Reg\n\
         operator fun Qty.get(i: Int) = i\n\
         operator fun Qty.component1() = 1\n\
         operator fun Bag.iterator(): Iterator<Int> = listOf(1).iterator()\n\
         operator fun Reg.getValue(t: Any?, p: Any?) = 1\n\
         operator fun Reg.setValue(t: Any?, p: Any?, v: Int) {}\n\
         fun Qty.set(i: Int) = i\n\
         fun Qty.component2() = 2\n",
    )]);
    let before = "package com.x.util\n\n\
        fun go(q: Qty, b: Bag, r: Reg): Int {\n\
        \x20   val (one) = q\n\
        \x20   var n by r\n\
        \x20   for (v in b) n += v\n\
        \x20   return q[0] + one + n\n\
        }\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, _) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    for name in ["get", "component1", "iterator", "getValue", "setValue"] {
        assert!(
            text.contains(&format!("import com.x.util.{name}\n")),
            "{name}: {text}"
        );
    }
    // Not an operator, and not used: `set` is a plain extension, and the
    // file destructures one element only.
    assert!(!text.contains("import com.x.util.set\n"), "{text}");
    assert!(!text.contains("import com.x.util.component2\n"), "{text}");
}

#[test]
fn a_file_that_only_indexes_a_list_gets_no_import_for_an_operator_it_has_no_use_for() {
    let project = Project::new(&[(
        &format!("{OLD}/Ops.kt"),
        "package com.x.util\n\nclass Qty\nfun Qty.get(i: Int) = i\nfun other() = 1\n",
    )]);
    // `get` is declared, but not as an operator: `list[0]` cannot mean it.
    let before = "package com.x.util\n\nfun go(l: List<Int>) = l[0]\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, notes) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    assert_eq!(text, after);
    assert!(notes.is_empty(), "{notes:?}");
}

/// The other direction: the file that declares the operators moves, the file
/// that stayed behind in the old package writes `q[0]` and `for (v in bag)`.
#[test]
fn a_file_left_behind_gets_the_imports_of_the_operators_that_moved_away() {
    let user = format!("{OLD}/Use.kt");
    let project = Project::new(&[
        (
            &user,
            "package com.x.util\n\nfun go(q: Qty, b: Bag): Int {\n    var t = q[0]\n    for (v in b) t += v\n    return t\n}\n",
        ),
        (
            &format!("{OLD}/Plain.kt"),
            "package com.x.util\n\nfun plain(l: List<Int>) = l[0]\n",
        ),
        (
            &format!("{OLD}/Other.kt"),
            "package com.x.other\n\nfun other(q: Qty) = q[0]\n",
        ),
    ]);
    let before = "package com.x.util\n\nclass Qty\nclass Bag\n\
        operator fun Qty.get(i: Int) = i\n\
        operator fun Bag.iterator(): Iterator<Int> = listOf(1).iterator()\n\
        operator fun Qty.plus(o: Qty) = this\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (written, notes) = project.repair_all(
        &format!("{OLD}/Qty.kt"),
        &format!("{NEW}/Qty.kt"),
        before,
        &after,
    );

    let of = |name: &str| {
        written
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .map(|(_, text)| text.as_str())
    };
    let used = of("Use.kt").expect("Use.kt got imports");
    assert!(used.contains("import com.x.util.sub.get\n"), "{used}");
    assert!(used.contains("import com.x.util.sub.iterator\n"), "{used}");
    // `plus` is written as a symbol: the server's job, not this layer's.
    assert!(!used.contains("plus"), "{used}");
    // A file that indexes a list and never writes `Qty` is not touched, and
    // another package never saw the operators.
    assert!(of("Plain.kt").is_none());
    assert!(of("Other.kt").is_none());
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn an_operator_of_another_type_is_not_imported_for_a_loop_an_index_or_by_lazy() {
    let project = Project::new(&[(
        &format!("{OLD}/Ops.kt"),
        "package com.x.util\n\nclass Qty\n\
         operator fun Qty.get(i: Int) = i\n\
         operator fun Qty.getValue(t: Any?, p: Any?) = 1\n\
         operator fun <T> Iterable<Qty>.iterator(): Iterator<T> = TODO()\n",
    )]);
    // None of these write `Qty` or `Iterable`, so none can mean those.
    let before = "package com.x.util\n\nval a by lazy { 1 }\n\
        fun go(l: List<Int>) = l[0]\nfun loop(l: List<Int>) { for (v in l) println(v) }\n";
    let after = before.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (text, notes) = project.repair(
        &format!("{OLD}/Moving.kt"),
        &format!("{NEW}/Moving.kt"),
        before,
        &after,
    );

    assert_eq!(text, after);
    assert!(notes.is_empty(), "{notes:?}");
}

const APP: &str = "app/src/commonMain/kotlin/com/x/util";
const APP_NEW: &str = "app/src/commonMain/kotlin/com/x/util/sub";
const LIB_OPS: &str =
    "package com.x.util\n\nclass Qty\noperator fun Qty.get(i: Int) = i\nfun Qty.helper() = 1\n";
const MOVING: &str = "package com.x.util\n\nfun go(q: Qty) = q[0] + q.helper()\n";

/// A package split over two modules: the library declares `Qty.get`, the app
/// uses it from the same package.
fn split(app_build: &str) -> Project {
    Project::new(&[
        ("app/build.gradle.kts", app_build),
        ("lib/build.gradle.kts", ""),
        ("lib/src/commonMain/kotlin/com/x/util/Ops.kt", LIB_OPS),
    ])
}

#[test]
fn an_operator_of_a_module_the_build_file_names_is_imported_and_a_plain_name_is_not() {
    for build in [
        "dependencies { implementation(project(\":lib\")) }",
        "dependencies { api(projects.lib) }",
    ] {
        let project = split(build);
        let after = MOVING.replace("package com.x.util\n", "package com.x.util.sub\n");

        let (text, _) = project.repair(
            &format!("{APP}/Moving.kt"),
            &format!("{APP_NEW}/Moving.kt"),
            MOVING,
            &after,
        );

        // `q[0]` is the syntax calling `get`; `q.helper()` is a name, and the
        // server imports the names of other modules by itself.
        assert!(text.contains("import com.x.util.get\n"), "{build}: {text}");
        assert!(
            !text.contains("import com.x.util.helper"),
            "{build}: {text}"
        );
    }
}

#[test]
fn an_operator_of_a_module_that_is_no_dependency_is_not_imported() {
    for build in [
        "",
        "dependencies { implementation(project(\":library\")) }",
        "dependencies { api(projects.library) }",
    ] {
        let project = split(build);
        let after = MOVING.replace("package com.x.util\n", "package com.x.util.sub\n");

        let (text, notes) = project.repair(
            &format!("{APP}/Moving.kt"),
            &format!("{APP_NEW}/Moving.kt"),
            MOVING,
            &after,
        );

        assert_eq!(text, after, "{build}");
        assert!(notes.is_empty(), "{build}: {notes:?}");
    }
}

#[test]
fn a_file_of_a_dependent_module_gets_the_operators_of_the_moved_declaring_file() {
    let project = Project::new(&[
        (
            "app/build.gradle.kts",
            "dependencies { implementation(project(\":lib\")) }",
        ),
        ("lib/build.gradle.kts", ""),
        (
            "app/src/commonMain/kotlin/com/x/util/Use.kt",
            "package com.x.util\n\nfun go(q: Qty) = q[0]\n",
        ),
    ]);
    let lib_old = "lib/src/commonMain/kotlin/com/x/util";
    let after = LIB_OPS.replace("package com.x.util\n", "package com.x.util.sub\n");

    let (written, _) = project.repair_all(
        &format!("{lib_old}/Qty.kt"),
        &format!("{lib_old}/sub/Qty.kt"),
        LIB_OPS,
        &after,
    );

    let used = written
        .iter()
        .find(|(path, _)| path.ends_with("Use.kt"))
        .map(|(_, text)| text.as_str())
        .expect("Use.kt got an import");
    assert!(used.contains("import com.x.util.sub.get\n"), "{used}");
}
