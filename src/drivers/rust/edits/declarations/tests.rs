//! Where the new declaration lands in the parent's file.

use super::*;

fn insert(content: &str, name: &str) -> String {
    let edit = insert_module_declaration(
        Path::new("mod.rs"),
        content,
        name,
        &format!("pub mod {name};"),
    );
    let mut result = content.to_string();
    result.replace_range(edit.start..edit.end, &edit.replacement);
    result
}

#[test]
fn a_new_declaration_joins_a_sorted_block_at_its_alphabetical_place() {
    let content = "pub mod alpha;\npub mod omega;\n\npub fn run() {}\n";
    assert_eq!(
        insert(content, "matching"),
        "pub mod alpha;\npub mod matching;\npub mod omega;\n\npub fn run() {}\n"
    );
}

#[test]
fn a_name_that_sorts_last_goes_after_the_last_declaration_not_at_the_end_of_the_file() {
    let content = "pub mod alpha;\n\npub fn run() {}\n\n#[cfg(test)]\nmod tests {}\n";
    assert_eq!(
        insert(content, "zeta"),
        "pub mod alpha;\npub mod zeta;\n\npub fn run() {}\n\n#[cfg(test)]\nmod tests {}\n"
    );
}

#[test]
fn an_unsorted_block_gets_the_declaration_after_its_last_line() {
    let content = "pub mod zeta;\npub mod alpha;\nfn x() {}\n";
    assert_eq!(
        insert(content, "matching"),
        "pub mod zeta;\npub mod alpha;\npub mod matching;\nfn x() {}\n"
    );
}

#[test]
fn a_file_without_declarations_gets_it_at_the_end() {
    assert_eq!(
        insert("fn x() {}\n", "matching"),
        "fn x() {}\npub mod matching;\n"
    );
    assert_eq!(insert("", "matching"), "pub mod matching;\n");
}

#[test]
fn inline_modules_are_not_declarations_and_a_missing_final_newline_is_kept_valid() {
    let content = "mod inline { fn x() {} }\npub mod alpha;";
    assert_eq!(
        insert(content, "beta"),
        "mod inline { fn x() {} }\npub mod alpha;\npub mod beta;\n"
    );
}

#[test]
fn a_tests_module_far_below_the_block_is_not_part_of_it() {
    let content = "pub mod alpha;\npub mod beta;\n\nuse std::fmt;\n\n#[cfg(test)]\nmod tests;\n";
    assert_eq!(
        insert(content, "gamma"),
        "pub mod alpha;\npub mod beta;\npub mod gamma;\n\nuse std::fmt;\n\n#[cfg(test)]\nmod tests;\n"
    );
}

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| part.to_string()).collect()
}

#[test]
fn a_private_declaration_in_a_deeper_parent_is_widened_to_the_old_parent() {
    let old = path(&["drivers", "lsp", "rename"]);
    assert_eq!(
        visibility_prefix("", &old, &path(&["drivers", "lsp", "rename", "plan"])),
        "pub(super) "
    );
    assert_eq!(
        visibility_prefix(
            "",
            &old,
            &path(&["drivers", "lsp", "rename", "plan", "deep"])
        ),
        "pub(in crate::drivers::lsp::rename) "
    );
    assert_eq!(
        visibility_prefix("", &old, &path(&["drivers", "lsp", "other"])),
        "pub(super) "
    );
    assert_eq!(
        visibility_prefix("", &old, &path(&["drivers", "lsp", "other", "deep"])),
        "pub(in crate::drivers::lsp) "
    );
    assert_eq!(
        visibility_prefix("", &old, &path(&["elsewhere"])),
        "pub(super) "
    );
    assert_eq!(
        visibility_prefix("", &old, &path(&["elsewhere", "deep"])),
        "pub(crate) "
    );
}

#[test]
fn a_private_declaration_that_stays_in_or_moves_above_its_old_parent_stays_private() {
    let old = path(&["drivers", "lsp", "rename"]);
    assert_eq!(visibility_prefix("", &old, &old), "");
    assert_eq!(visibility_prefix("", &old, &path(&["drivers", "lsp"])), "");
    assert_eq!(visibility_prefix("", &old, &[]), "");
}

#[test]
fn a_written_visibility_is_kept_as_it_is() {
    let old = path(&["a"]);
    let new = path(&["b", "c"]);
    assert_eq!(visibility_prefix("pub", &old, &new), "pub ");
    assert_eq!(visibility_prefix("pub(crate)", &old, &new), "pub(crate) ");
}

#[test]
fn pub_super_and_pub_self_are_measured_from_where_they_reached() {
    let old = path(&["drivers", "lsp", "rename"]);
    // `pub(super)` reached `lsp`; in a deeper parent it has to say so.
    assert_eq!(
        visibility_prefix(
            "pub(super)",
            &old,
            &path(&["drivers", "lsp", "rename", "plan"])
        ),
        "pub(in crate::drivers::lsp) "
    );
    assert_eq!(
        visibility_prefix("pub(super)", &old, &path(&["drivers", "lsp", "other"])),
        "pub(super) "
    );
    assert_eq!(visibility_prefix("pub(self)", &old, &old), "");
    assert_eq!(
        visibility_prefix(
            "pub(self)",
            &old,
            &path(&["drivers", "lsp", "rename", "plan"])
        ),
        "pub(super) "
    );
}
