use super::*;

const MOVED: &str = "package com.example.droid.ui

import android.app.Activity
import com.example.droid.widgets.BadgeView

class MainActivity : Activity() {
    fun title() = getString(R.string.app_name)
}
";

#[test]
fn adds_the_missing_import_in_sorted_position() {
    let updated = add_generated_imports(MOVED, "com.example.droid").unwrap();
    assert!(updated.contains(
        "import android.app.Activity\nimport com.example.droid.R\nimport com.example.droid.widgets.BadgeView\n"
    ));
}

#[test]
fn adds_build_config_and_r_when_both_are_used() {
    let text = "package a.b\n\nimport z.Z\n\nval x = R.id.main + BuildConfig.VERSION_CODE\n";
    let updated = add_generated_imports(text, "com.example").unwrap();
    assert!(updated.contains("import com.example.BuildConfig\n"));
    assert!(updated.contains("import com.example.R\n"));
    assert!(updated.contains("import z.Z\n"));
}

#[test]
fn nothing_is_added_when_the_file_does_not_use_the_class_or_imports_it() {
    let unqualified = "package a\n\nval x = android.R.id.home\nval y = Other.R.id\nval z = SR.x\n// R.string.name\n";
    assert_eq!(add_generated_imports(unqualified, "n.s"), None);
    let imported = "package a\n\nimport android.R\n\nval x = R.id.home\n";
    assert_eq!(add_generated_imports(imported, "n.s"), None);
    let already = "package a\n\nimport n.s.R\n\nval x = R.id.home\n";
    assert_eq!(add_generated_imports(already, "n.s"), None);
}

#[test]
fn a_file_without_imports_gets_a_block_below_the_package() {
    let text = "package a.b\n\nval x = R.id.main\n";
    assert_eq!(
        add_generated_imports(text, "n.s").unwrap(),
        "package a.b\n\nimport n.s.R\n\nval x = R.id.main\n"
    );
}

#[test]
fn unsorted_imports_get_the_new_one_after_the_last() {
    let text = "package a\n\nimport z.Z\nimport b.B\n\nval x = R.id.main\n";
    let updated = add_generated_imports(text, "n.s").unwrap();
    assert!(updated.contains("import z.Z\nimport b.B\nimport n.s.R\n"));
}

#[test]
fn windows_line_endings_are_kept() {
    let text = "package a\r\n\r\nimport z.Z\r\n\r\nval x = R.id.main\r\n";
    let updated = add_generated_imports(text, "n.s").unwrap();
    assert!(updated.contains("import n.s.R\r\n"));
    assert!(!updated.replace("\r\n", "").contains('\n'));
}
