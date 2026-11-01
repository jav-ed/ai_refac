//! Calls that select files by pattern, and the mentions that are not calls.

use super::*;

/// A glob names a folder and a pattern, never the files it selects.
#[test]
fn a_project_with_a_glob_uses_every_moved_file() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/Team.astro",
        "---\nconst photos = import.meta.glob(\"../assets/team/*.webp\");\n---\n",
    );
    write(dir.path(), "assets/team/ada.webp", "");
    write(dir.path(), "assets/other/lin.webp", "");

    let used = used_by_code(
        &[pair("assets/team/ada.webp"), pair("assets/other/lin.webp")],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true]);
}

// Rule: a glob only counts where code can call it. A docs folder that
// explains `import.meta.glob(...)` in prose, in a fenced or inline example, or
// that writes the bare name, must not start the TypeScript engine for every
// Markdown move (the engine then parses unrelated files and can fail on them).
// Benefit 8/10: a Markdown-only move in a docs repo broke on this.
#[test]
fn a_glob_written_as_an_example_or_a_bare_name_is_not_a_call() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "docs/guide.md",
        "Use `import.meta.glob(\"./x/*.md\")` to list pages.\n",
    );
    write(
        dir.path(),
        "docs/fenced.mdx",
        "Example:\n\n```ts\nconst a = import.meta.glob(\"./x/*.md\");\n```\n\nand `Astro.glob(\"./y\")` too.\n",
    );
    write(
        dir.path(),
        "src/note.ts",
        "// import.meta.glob is not used in this file\nexport const x = 1;\n",
    );
    write(dir.path(), "assets/a.png", "");

    let used = used_by_code(&[pair("assets/a.png")], Some(dir.path())).unwrap();

    assert_eq!(used, [false]);
}

// Rule: the fence check must not hide a real call. A call in an `.mdx` page
// outside any fence, and one after a closed fence, still use every moved file.
// Benefit 7/10: a false "not a call" leaves a glob stale after a move.
#[test]
fn a_glob_call_in_an_mdx_page_outside_a_fence_is_still_a_call() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "docs/list.mdx",
        "```ts\nimport.meta.glob(\"./example\");\n```\n\nexport const pages = import.meta.glob<string>(\"./*.md\");\n",
    );
    write(dir.path(), "assets/a.png", "");

    let used = used_by_code(&[pair("assets/a.png")], Some(dir.path())).unwrap();

    assert_eq!(used, [true]);
}
