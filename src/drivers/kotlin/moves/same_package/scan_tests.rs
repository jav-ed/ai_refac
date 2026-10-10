//! The scanners (declarations, used names, imports) and the source-set rule.

use super::declared::{Kind, declarations};
use super::imports::Imports;
use super::used::used_names;
use super::visible;

fn names(text: &str) -> Vec<(String, Kind, bool)> {
    declarations(text)
        .into_iter()
        .map(|d| (d.name, d.kind, d.extension))
        .collect()
}

#[test]
fn every_kind_of_top_level_declaration_is_read_with_its_modifiers_and_annotations() {
    let text = "package a\n\n\
        fun shout(s: String) = s\n\
        @Composable fun Inline() {}\n\
        @Suppress(\"X Y\") internal fun quiet() {}\n\
        open class Base\n\
        abstract class Shape\n\
        infix fun Int.plusOne(o: Int) = this\n\
        expect fun platformName(): String\n\
        actual fun other(): String = \"\"\n\
        const val LIMIT = 3\n\
        val <T> List<T>.second: T get() = this[1]\n\
        fun <T> Map<String, List<T>>.firstOf(): T? = null\n\
        fun interface Callback { fun call() }\n\
        typealias Names = List<String>\n\
        private fun hidden() {}\n\
        private class Hidden\n\
        data object Single\n";
    let found: Vec<String> = names(text).into_iter().map(|d| d.0).collect();
    assert_eq!(
        found,
        [
            "shout",
            "Inline",
            "quiet",
            "Base",
            "Shape",
            "plusOne",
            "platformName",
            "other",
            "LIMIT",
            "second",
            "firstOf",
            "Callback",
            "Names",
            "Single"
        ]
    );
    let kinds = names(text);
    assert_eq!(kinds[0], ("shout".to_string(), Kind::Callable, false));
    assert_eq!(kinds[3], ("Base".to_string(), Kind::Type, false));
    assert_eq!(kinds[5], ("plusOne".to_string(), Kind::Callable, true));
}

#[test]
fn nothing_inside_comments_raw_strings_or_bodies_is_a_top_level_declaration() {
    let text = "/* fun inComment() {}\nfun alsoInComment() {}\n*/\n\
        val text = \"/*\"\nfun real() {}\n\
        val raw = \"\"\"\nfun inRaw() {}\n\"\"\"\n\
        // fun commented() {}\n\
        class Holder {\n    fun member() {}\n}\n";
    let found: Vec<String> = names(text).into_iter().map(|d| d.0).collect();
    assert_eq!(found, ["text", "real", "raw", "Holder"]);
}

#[test]
fn names_are_used_outside_comments_and_strings_but_inside_templates() {
    let text = "package a\nimport b.Imported\n\
        // inComment()\n/* nested /* inBlock() */ still */\n\
        fun f() {\n\
            val a = shout(\"inString()\") + \"$templated and ${braced(1)}\"\n\
            helper.member().chain()\n\
            val r = 1..limit\n\
            @Composable\n\
            val c = 'x'\n\
            val raw = \"\"\"inRaw() $inRawTemplate\"\"\"\n\
        }\n";
    let used = used_names(text);
    let plain: Vec<&str> = used.plain.iter().map(String::as_str).collect();
    assert!(plain.contains(&"shout"), "{plain:?}");
    assert!(plain.contains(&"templated"), "{plain:?}");
    assert!(plain.contains(&"braced"), "{plain:?}");
    assert!(plain.contains(&"helper"), "{plain:?}");
    assert!(
        plain.contains(&"limit"),
        "a range `1..limit` is not member access"
    );
    assert!(plain.contains(&"Composable"), "{plain:?}");
    assert!(plain.contains(&"inRawTemplate"), "{plain:?}");
    for hidden in ["inComment", "inBlock", "inString", "inRaw"] {
        assert!(
            !plain.contains(&hidden),
            "{hidden} is in a comment or a string"
        );
    }
    let member: Vec<&str> = used.member.iter().map(String::as_str).collect();
    assert_eq!(member, ["chain", "member"]);
}

#[test]
fn imports_know_aliases_stars_and_every_package_of_a_simple_name() {
    let imports = Imports::parse(
        "package a\nimport b.c.Helper\nimport d.e.Long as Short\nimport f.*\nimport g.h.`odd`;\n",
    );
    assert!(imports.binds("Helper") && imports.binds("Short") && imports.binds("odd"));
    assert!(!imports.binds("Long") && !imports.binds("f"));
    assert!(imports.star("f") && !imports.star("g"));
}

#[test]
fn source_sets_see_their_main_set_and_common_code_but_not_each_other() {
    assert!(visible(Some("main"), Some("main")));
    assert!(visible(Some("test"), Some("main")));
    assert!(visible(Some("jvmMain"), Some("commonMain")));
    assert!(visible(Some("jvmTest"), Some("jvmMain")));
    assert!(visible(Some("jvmTest"), Some("commonTest")));
    assert!(visible(Some("debug"), Some("main")));
    assert!(!visible(Some("main"), Some("test")));
    assert!(!visible(Some("commonMain"), Some("jvmMain")));
    assert!(!visible(Some("jvmMain"), Some("androidMain")));
    assert!(!visible(Some("jvmMain"), Some("jvmTest")));
    assert!(visible(None, Some("jvmMain")) && visible(Some("main"), None));
}
