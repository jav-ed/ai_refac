//! Exported symbols renamed across every import form.

use super::{assert_succeeded, assert_typechecks, rename};
use crate::common;

#[tokio::test]
async fn renames_exported_const_across_every_import_form() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_typechecks(project).await;
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1"],
    ));

    let util = common::read_file(project, "src/lib/util.ts");
    assert!(
        util.contains("export const grandTotal = 10;"),
        "declaration:\n{util}"
    );
    assert!(
        util.contains("const total = items.reduce"),
        "shadowing local untouched:\n{util}"
    );

    let a = common::read_file(project, "src/a.ts");
    assert!(
        a.contains("import { grandTotal, computeSum as sum,"),
        "named import:\n{a}"
    );
    assert!(
        a.contains("export { grandTotal as total } from"),
        "re-export keeps its public name:\n{a}"
    );
    assert!(a.contains("const local = grandTotal + 1;"), "usage:\n{a}");
    assert!(
        a.contains("{ total: grandTotal }"),
        "shorthand property expanded:\n{a}"
    );
    assert!(
        a.contains("util.grandTotal"),
        "namespace member access:\n{a}"
    );
    assert!(
        a.contains("function shadow(total: number) { return total * 2; }"),
        "shadowing parameter untouched:\n{a}"
    );
    assert!(
        a.contains("\"total in string\"") && a.contains("// total in comment"),
        "strings and comments untouched:\n{a}"
    );

    let b = common::read_file(project, "src/b.ts");
    assert!(
        b.contains("const { grandTotal: total } = await import"),
        "dynamic import destructuring:\n{b}"
    );
    let d = common::read_file(project, "src/d.js");
    assert!(
        d.contains("import { grandTotal } from") && d.contains("grandTotal + 1"),
        "javascript file:\n{d}"
    );
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_class_property_from_a_usage_site() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/a.ts", "count", "value", &[]));

    assert!(common::read_file(project, "src/lib/util.ts").contains("    value = 0;"));
    assert!(common::read_file(project, "src/lib/util.ts").contains("this.value++"));
    assert!(common::read_file(project, "src/a.ts").contains("c.value"));
    assert!(common::read_file(project, "src/c.tsx").contains("props.counter.value"));
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_default_exported_function_and_its_importers() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "defaultFn",
        "mainFn",
        &[],
    ));

    assert!(
        common::read_file(project, "src/lib/util.ts").contains("export default function mainFn()")
    );
    let a = common::read_file(project, "src/a.ts");
    assert!(
        a.contains("import mainFn from") && a.contains("mainFn()"),
        "default import follows:\n{a}"
    );
    assert_typechecks(project).await;
}

#[tokio::test]
async fn renames_local_export_without_changing_the_public_name() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/a.ts", "local", "localTotal", &[]));

    let a = common::read_file(project, "src/a.ts");
    assert!(a.contains("const localTotal = total + 1;"), "{a}");
    assert!(
        a.contains("export { localTotal as local,"),
        "importers of `local` keep working:\n{a}"
    );
    assert_typechecks(project).await;
}
