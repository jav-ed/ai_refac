use super::*;

fn renames(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(old, new)| (old.to_string(), new.to_string()))
        .collect()
}

fn apply(text: &str, pairs: &[(&str, &str)], is_manifest: bool) -> Option<String> {
    let renames = renames(pairs);
    let names = Names {
        namespace: "com.example.droid",
        renames: &renames,
        is_manifest,
    };
    rewrite(text, &names).unwrap()
}

const MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application>
        <!-- <activity android:name=".MainActivity" /> -->
        <activity android:name=".MainActivity" android:exported="true" />
        <service android:name="com.example.droid.sync.SyncService" />
        <receiver android:name='Boot' />
    </application>
</manifest>
"#;

#[test]
fn a_relative_name_stays_relative_while_it_is_under_the_namespace() {
    let updated = apply(
        MANIFEST,
        &[(
            "com.example.droid.MainActivity",
            "com.example.droid.ui.MainActivity",
        )],
        true,
    )
    .unwrap();
    assert!(
        updated.contains(r#"<activity android:name=".ui.MainActivity" android:exported="true" />"#)
    );
    // Commented-out XML is left alone.
    assert!(updated.contains(r#"<!-- <activity android:name=".MainActivity" /> -->"#));
}

#[test]
fn a_class_leaving_the_namespace_becomes_fully_qualified() {
    let updated = apply(
        MANIFEST,
        &[("com.example.droid.MainActivity", "org.other.MainActivity")],
        true,
    )
    .unwrap();
    assert!(updated.contains(r#"android:name="org.other.MainActivity" android:exported"#));
}

#[test]
fn qualified_and_bare_manifest_names_are_renamed() {
    let updated = apply(
        MANIFEST,
        &[
            (
                "com.example.droid.sync.SyncService",
                "com.example.droid.work.SyncService",
            ),
            ("com.example.droid.Boot", "com.example.droid.boot.Boot"),
        ],
        true,
    )
    .unwrap();
    assert!(updated.contains(r#"android:name="com.example.droid.work.SyncService""#));
    assert!(updated.contains(r#"android:name='.boot.Boot'"#));
    // A bare name only means something in the manifest.
    let layout = r#"<TextView android:name="Boot" />"#;
    let boot = [("com.example.droid.Boot", "com.example.droid.boot.Boot")];
    assert_eq!(apply(layout, &boot, false), None);
}

#[test]
fn custom_view_elements_and_tools_context_are_renamed() {
    let layout = r#"<LinearLayout xmlns:tools="http://schemas.android.com/tools"
    tools:context=".MainActivity">
    <com.example.droid.widgets.BadgeView
        android:id="@+id/badge" />
</LinearLayout>
"#;
    let updated = apply(
        layout,
        &[
            (
                "com.example.droid.widgets.BadgeView",
                "com.example.droid.ui.BadgeView",
            ),
            (
                "com.example.droid.MainActivity",
                "com.example.droid.ui.MainActivity",
            ),
        ],
        false,
    )
    .unwrap();
    assert!(updated.contains("<com.example.droid.ui.BadgeView"));
    assert!(updated.contains(r#"tools:context=".ui.MainActivity""#));
    let paired = "<com.example.droid.widgets.BadgeView></com.example.droid.widgets.BadgeView>";
    let both = apply(
        paired,
        &[("com.example.droid.widgets.BadgeView", "a.B")],
        false,
    )
    .unwrap();
    assert_eq!(both, "<a.B></a.B>");
}

#[test]
fn navigation_destinations_and_argument_types_are_renamed() {
    let graph = r#"<navigation>
    <fragment android:id="@+id/home" android:name="com.example.droid.ui.HomeFragment">
        <argument android:name="item" app:argType="com.example.droid.model.Item" />
    </fragment>
</navigation>"#;
    let updated = apply(
        graph,
        &[
            (
                "com.example.droid.ui.HomeFragment",
                "com.example.droid.home.HomeFragment",
            ),
            (
                "com.example.droid.model.Item",
                "com.example.droid.data.Item",
            ),
        ],
        false,
    )
    .unwrap();
    assert!(updated.contains(r#"android:name="com.example.droid.home.HomeFragment""#));
    assert!(updated.contains(r#"app:argType="com.example.droid.data.Item""#));
}

#[test]
fn unrelated_files_are_not_touched_and_formatting_survives() {
    let text = "<a>\r\n  <b c=\"d\"/>\r\n</a>\r\n";
    assert_eq!(apply(text, &[("x.Y", "x.Z")], false), None);
    let with_name = "<a>\r\n  <x.Y/>\r\n</a>\r\n";
    assert_eq!(
        apply(with_name, &[("x.Y", "x.Z")], false).unwrap(),
        "<a>\r\n  <x.Z/>\r\n</a>\r\n"
    );
}

#[test]
fn malformed_xml_is_an_error() {
    let renames = renames(&[("x.Y", "x.Z")]);
    let names = Names {
        namespace: "n",
        renames: &renames,
        is_manifest: false,
    };
    assert!(rewrite(r#"<a b="c>"#, &names).is_err());
    assert!(rewrite("<a b=c/>", &names).is_err());
    assert!(rewrite("<a b", &names).is_err());
    assert!(rewrite("<!-- never closed <a/>", &names).is_err());
}
